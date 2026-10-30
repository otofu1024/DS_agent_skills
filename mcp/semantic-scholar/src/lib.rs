pub mod server;

pub use server::SemanticScholarServer;

use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, Url, header::RETRY_AFTER};
use rmcp::schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

pub const API_BASE: &str = "https://api.semanticscholar.org/graph/v1";
pub const USER_AGENT: &str =
    "ds-semantic-scholar-mcp/0.1 (+https://github.com/otofu1024/DS_agent_skills)";

pub const SEARCH_FIELDS: &str = "paperId,externalIds,url,title,abstract,venue,year,publicationTypes,authors,citationCount,openAccessPdf";
pub const PAPER_FIELDS: &str = "paperId,externalIds,url,title,abstract,venue,year,publicationTypes,authors,citationCount,openAccessPdf,publicationDate,influentialCitationCount,referenceCount,fieldsOfStudy,tldr";
pub const LINK_FIELDS: &str = "paperId,externalIds,url,title,venue,year,authors,citationCount";
pub const AUTHOR_FIELDS: &str =
    "authorId,name,url,affiliations,homepage,paperCount,citationCount,hIndex";
pub const AUTHOR_PAPER_FIELDS: &str = "paperId,externalIds,title,venue,year,citationCount";

pub const MAX_SEARCH_LIMIT: i64 = 100;
pub const MAX_SEARCH_OFFSET: i64 = 999;
pub const MAX_LINK_LIMIT: i64 = 100;
pub const MAX_AUTHORS: usize = 10;
pub const SEARCH_ABSTRACT_CHARS: usize = 1500;
pub const MAX_CONTEXTS: usize = 3;
pub const CONTEXT_CHARS: usize = 300;
pub const DEFAULT_INTERVAL: f64 = 1.1;
pub const MAX_RETRY_AFTER: f64 = 30.0;
const DEFAULT_MAX_RETRIES: usize = 3;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_STATUSES: [u16; 5] = [429, 500, 502, 503, 504];
const OMITTED: &str = "…（省略）";

const HINT_400: &str = "パラメータが不正です。error の内容を確認してください。";
const HINT_403: &str = "API キーが拒否されました。SEMANTIC_SCHOLAR_API_KEY を確認してください。";
const HINT_404: &str = "見つかりませんでした。ID の形式（S2 の paperId、DOI:...、ARXIV:...、CorpusId:...）を確認してください。";
const HINT_429: &str = "レート制限に達しました。少し待ってから再試行してください。";
const HINT_OTHER: &str = "Semantic Scholar 側のエラーです。時間をおいて再試行してください。";
const NO_KEY_429: &str =
    " API キーなしの共有枠は混みやすいので、SEMANTIC_SCHOLAR_API_KEY の設定を勧めます。";

fn search_limit_default() -> i64 {
    10
}

fn search_offset_default() -> i64 {
    0
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct SearchPapersArgs {
    #[schemars(description = "検索語。専門用語を使った英語の表現が最もよく当たる。")]
    pub query: String,
    #[serde(default = "search_limit_default")]
    #[schemars(
        default = "search_limit_default",
        description = "返す件数（1〜100、既定 10）。"
    )]
    pub limit: i64,
    #[serde(default = "search_offset_default")]
    #[schemars(
        default = "search_offset_default",
        description = "何件目から返すか（ページ送り用）。"
    )]
    pub offset: i64,
    #[serde(default)]
    #[schemars(
        description = "出版年の絞り込み。\"2019\"、\"2016-2020\"、\"2020-\"、\"-2010\" の形式。"
    )]
    pub year: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "会議・論文誌名の絞り込み（カンマ区切り）。表記揺れが多いので、外して検索してから要旨で選ぶ方が確実。"
    )]
    pub venue: Option<String>,
    #[serde(default)]
    #[schemars(description = "分野の絞り込み（例: \"Computer Science\"）。")]
    pub fields_of_study: Option<String>,
    #[serde(default)]
    #[schemars(description = "被引用数の下限。")]
    pub min_citation_count: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub api_key: Option<String>,
    pub interval_seconds: f64,
    pub max_retries: usize,
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            interval_seconds: DEFAULT_INTERVAL,
            max_retries: DEFAULT_MAX_RETRIES,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl ClientConfig {
    pub fn from_env() -> Result<Self, std::num::ParseFloatError> {
        let api_key = env::var("SEMANTIC_SCHOLAR_API_KEY").unwrap_or_default();
        let interval = env::var("S2_MIN_INTERVAL").ok();
        let max_retries = env::var("S2_MAX_RETRIES")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_RETRIES)
            .min(DEFAULT_MAX_RETRIES);
        let mut config = Self::from_values(Some(&api_key), interval.as_deref())?;
        config.max_retries = max_retries;
        Ok(config)
    }

    pub fn from_values(
        api_key: Option<&str>,
        interval: Option<&str>,
    ) -> Result<Self, std::num::ParseFloatError> {
        let api_key = api_key
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .map(ToOwned::to_owned);
        let interval_seconds = interval.map_or(Ok(DEFAULT_INTERVAL), str::parse::<f64>)?;
        Ok(Self {
            api_key,
            interval_seconds,
            ..Self::default()
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub retry_after: Option<String>,
}

#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn fetch(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<HttpResponse, String>;
}

#[async_trait]
pub trait Sleeper: Send + Sync {
    async fn sleep(&self, seconds: f64);
}

pub trait Clock: Send + Sync {
    fn now(&self) -> f64;
}

pub struct ReqwestTransport {
    client: ReqwestClient,
}

impl ReqwestTransport {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: ReqwestClient::builder().build()?,
        })
    }
}

#[async_trait]
impl HttpTransport for ReqwestTransport {
    async fn fetch(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<HttpResponse, String> {
        let mut request = self.client.get(url).timeout(timeout);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        let response = request.send().await.map_err(|error| error.to_string())?;
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let body = response.bytes().await.map_err(|error| error.to_string())?;
        Ok(HttpResponse {
            status,
            body: String::from_utf8_lossy(&body).into_owned(),
            retry_after,
        })
    }
}

pub struct TokioSleeper;

#[async_trait]
impl Sleeper for TokioSleeper {
    async fn sleep(&self, seconds: f64) {
        if seconds.is_finite() && seconds > 0.0 {
            tokio::time::sleep(Duration::from_secs_f64(seconds)).await;
        }
    }
}

pub struct MonotonicClock {
    epoch: Instant,
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl Clock for MonotonicClock {
    fn now(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64()
    }
}

struct RateLimiter {
    interval_seconds: f64,
    last_start: Mutex<Option<f64>>,
    clock: Arc<dyn Clock>,
    sleeper: Arc<dyn Sleeper>,
}

impl RateLimiter {
    fn new(interval_seconds: f64, clock: Arc<dyn Clock>, sleeper: Arc<dyn Sleeper>) -> Self {
        Self {
            interval_seconds,
            last_start: Mutex::new(None),
            clock,
            sleeper,
        }
    }

    async fn wait(&self) {
        let mut last_start = self.last_start.lock().await;
        if let Some(previous) = *last_start {
            let delay = previous + self.interval_seconds - self.clock.now();
            if delay > 0.0 {
                self.sleeper.sleep(delay).await;
            }
        }
        *last_start = Some(self.clock.now());
    }
}

#[derive(Debug)]
enum ClientError {
    Api { status: u16, message: String },
    Transport(String),
    InvalidJson(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api { status, message } => write!(formatter, "HTTP {status}: {message}"),
            Self::Transport(message) | Self::InvalidJson(message) => formatter.write_str(message),
        }
    }
}

impl Error for ClientError {}

#[derive(Clone)]
pub struct SemanticScholarClient {
    api_key: Option<String>,
    timeout: Duration,
    max_retries: usize,
    transport: Arc<dyn HttpTransport>,
    sleeper: Arc<dyn Sleeper>,
    limiter: Arc<RateLimiter>,
}

impl SemanticScholarClient {
    pub fn new(config: ClientConfig) -> Result<Self, reqwest::Error> {
        let transport = Arc::new(ReqwestTransport::new()?);
        Ok(Self::with_dependencies(
            config,
            transport,
            Arc::new(MonotonicClock::default()),
            Arc::new(TokioSleeper),
        ))
    }

    pub fn from_env() -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self::new(ClientConfig::from_env()?)?)
    }

    pub fn with_dependencies(
        config: ClientConfig,
        transport: Arc<dyn HttpTransport>,
        clock: Arc<dyn Clock>,
        sleeper: Arc<dyn Sleeper>,
    ) -> Self {
        let limiter = Arc::new(RateLimiter::new(
            config.interval_seconds,
            clock,
            Arc::clone(&sleeper),
        ));
        Self {
            api_key: config.api_key,
            timeout: config.timeout,
            max_retries: config.max_retries,
            transport,
            sleeper,
            limiter,
        }
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    async fn get(
        &self,
        path: &str,
        params: &[(&str, Option<String>)],
    ) -> Result<Value, ClientError> {
        let url = build_url(path, params)?;
        let mut headers = HashMap::from([
            ("User-Agent".to_owned(), USER_AGENT.to_owned()),
            ("Accept".to_owned(), "application/json".to_owned()),
        ]);
        if let Some(api_key) = &self.api_key {
            headers.insert("x-api-key".to_owned(), api_key.clone());
        }

        for attempt in 0..=self.max_retries {
            self.limiter.wait().await;
            let response = self
                .transport
                .fetch(&url, &headers, self.timeout)
                .await
                .map_err(ClientError::Transport)?;
            if response.status == 200 {
                return serde_json::from_str(&response.body)
                    .map_err(|error| ClientError::InvalidJson(error.to_string()));
            }
            if RETRY_STATUSES.contains(&response.status) && attempt < self.max_retries {
                self.sleeper
                    .sleep(backoff(response.retry_after.as_deref(), attempt))
                    .await;
                continue;
            }
            return Err(ClientError::Api {
                status: response.status,
                message: error_message(&response.body),
            });
        }
        unreachable!("retry loop always returns or continues through its final attempt")
    }

    pub async fn search_papers(&self, args: SearchPapersArgs) -> Value {
        let SearchPapersArgs {
            query,
            limit,
            offset,
            year,
            venue,
            fields_of_study,
            min_citation_count,
        } = args;
        let limit = clamp(limit, 1, MAX_SEARCH_LIMIT);
        let offset = clamp(offset, 0, MAX_SEARCH_OFFSET);
        let params = [
            ("query", Some(query)),
            ("limit", Some(limit.to_string())),
            ("offset", Some(offset.to_string())),
            ("fields", Some(SEARCH_FIELDS.to_owned())),
            ("year", year),
            ("venue", venue),
            ("fieldsOfStudy", fields_of_study),
            (
                "minCitationCount",
                min_citation_count.map(|count| count.to_string()),
            ),
        ];
        match self.get("/paper/search", &params).await {
            Ok(data) => {
                let papers = data
                    .get("data")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .map(|paper| format_paper(paper, Some(SEARCH_ABSTRACT_CHARS)))
                    .collect::<Vec<_>>();
                compact(json!({
                    "total": data.get("total").cloned().unwrap_or(Value::Null),
                    "offset": data.get("offset").cloned().unwrap_or(Value::Null),
                    "next": data.get("next").cloned().unwrap_or(Value::Null),
                    "papers": papers,
                }))
            }
            Err(error) => self.error_json(error),
        }
    }

    pub async fn match_paper(&self, title: String) -> Value {
        let params = [
            ("query", Some(title)),
            ("fields", Some(SEARCH_FIELDS.to_owned())),
        ];
        match self.get("/paper/search/match", &params).await {
            Ok(data) => {
                let first = data
                    .get("data")
                    .and_then(Value::as_array)
                    .and_then(|items| items.first());
                match first {
                    Some(paper) => json!({
                        "match": format_paper(paper, Some(SEARCH_ABSTRACT_CHARS))
                    }),
                    None => match_not_found(),
                }
            }
            Err(ClientError::Api { status: 404, .. }) => match_not_found(),
            Err(error) => self.error_json(error),
        }
    }

    pub async fn get_paper(&self, paper_id: String) -> Value {
        let path = format!("/paper/{}", paper_path(&paper_id));
        let params = [("fields", Some(PAPER_FIELDS.to_owned()))];
        match self.get(&path, &params).await {
            Ok(paper) => format_paper(&paper, None),
            Err(error) => self.error_json(error),
        }
    }

    pub async fn get_citations(&self, paper_id: String, limit: i64, offset: i64) -> Value {
        self.get_links(paper_id, "citations", "citingPaper", limit, offset)
            .await
    }

    pub async fn get_references(&self, paper_id: String, limit: i64, offset: i64) -> Value {
        self.get_links(paper_id, "references", "citedPaper", limit, offset)
            .await
    }

    async fn get_links(
        &self,
        paper_id: String,
        kind: &'static str,
        paper_key: &'static str,
        limit: i64,
        offset: i64,
    ) -> Value {
        let limit = clamp(limit, 1, MAX_LINK_LIMIT);
        let offset = offset.max(0);
        let path = format!("/paper/{}/{kind}", paper_path(&paper_id));
        let fields = format!("contexts,intents,isInfluential,{LINK_FIELDS}");
        let params = [
            ("fields", Some(fields)),
            ("limit", Some(limit.to_string())),
            ("offset", Some(offset.to_string())),
        ];
        match self.get(&path, &params).await {
            Ok(data) => {
                let links = data
                    .get("data")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .map(|item| format_link(item, paper_key))
                    .collect::<Vec<_>>();
                let mut output = Map::new();
                output.insert(
                    "offset".to_owned(),
                    data.get("offset").cloned().unwrap_or(Value::Null),
                );
                output.insert(
                    "next".to_owned(),
                    data.get("next").cloned().unwrap_or(Value::Null),
                );
                output.insert(kind.to_owned(), Value::Array(links));
                compact(Value::Object(output))
            }
            Err(error) => self.error_json(error),
        }
    }

    pub async fn get_author(&self, author_id: String, paper_limit: i64) -> Value {
        let paper_limit = clamp(paper_limit, 0, MAX_LINK_LIMIT);
        let path = format!("/author/{}", quote_path(&author_id, ""));
        let params = [("fields", Some(AUTHOR_FIELDS.to_owned()))];
        let author = match self.get(&path, &params).await {
            Ok(author) => author,
            Err(error) => return self.error_json(error),
        };
        let mut result = compact(json!({
            "authorId": author.get("authorId").cloned().unwrap_or(Value::Null),
            "name": author.get("name").cloned().unwrap_or(Value::Null),
            "affiliations": author.get("affiliations").cloned().unwrap_or(Value::Null),
            "homepage": author.get("homepage").cloned().unwrap_or(Value::Null),
            "paperCount": author.get("paperCount").cloned().unwrap_or(Value::Null),
            "citationCount": author.get("citationCount").cloned().unwrap_or(Value::Null),
            "hIndex": author.get("hIndex").cloned().unwrap_or(Value::Null),
            "url": author.get("url").cloned().unwrap_or(Value::Null),
        }));
        if paper_limit > 0 {
            let papers_path = format!("{path}/papers");
            let paper_params = [
                ("fields", Some(AUTHOR_PAPER_FIELDS.to_owned())),
                ("limit", Some(paper_limit.to_string())),
            ];
            let papers = match self.get(&papers_path, &paper_params).await {
                Ok(papers) => papers,
                Err(error) => return self.error_json(error),
            };
            let formatted = papers
                .get("data")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|paper| format_paper(paper, None))
                .collect::<Vec<_>>();
            if let Value::Object(object) = &mut result {
                object.insert("papers".to_owned(), Value::Array(formatted));
            }
        }
        result
    }

    fn error_json(&self, error: ClientError) -> Value {
        match error {
            ClientError::Api { status, message } => {
                let mut hint = match status {
                    400 => HINT_400.to_owned(),
                    403 => HINT_403.to_owned(),
                    404 => HINT_404.to_owned(),
                    429 => HINT_429.to_owned(),
                    _ => HINT_OTHER.to_owned(),
                };
                if status == 429 && !self.has_api_key() {
                    hint.push_str(NO_KEY_429);
                }
                json!({"error": message, "status": status, "hint": hint})
            }
            ClientError::Transport(message) => {
                json!({"error": format!("Semantic Scholar に接続できませんでした: {message}")})
            }
            ClientError::InvalidJson(message) => {
                json!({"error": format!("応答を解釈できませんでした: {message}")})
            }
        }
    }
}

fn build_url(path: &str, params: &[(&str, Option<String>)]) -> Result<String, ClientError> {
    let mut url = Url::parse(&format!("{API_BASE}{path}"))
        .map_err(|error| ClientError::Transport(error.to_string()))?;
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in params {
            if let Some(value) = value {
                pairs.append_pair(key, value);
            }
        }
    }
    Ok(url.into())
}

fn backoff(retry_after: Option<&str>, attempt: usize) -> f64 {
    if let Some(value) = retry_after.and_then(|value| value.parse::<f64>().ok()) {
        return value.clamp(0.0, MAX_RETRY_AFTER);
    }
    2_f64.powi((attempt + 1) as i32)
}

fn error_message(body: &str) -> String {
    let Ok(data) = serde_json::from_str::<Value>(body) else {
        return take_chars(body, 200);
    };
    if let Some(object) = data.as_object() {
        for key in ["message", "error"] {
            if let Some(value) = object.get(key).filter(|value| is_truthy(value)) {
                return take_chars(&python_string(value), 200);
            }
        }
        return take_chars(&python_repr(&data), 200);
    }
    take_chars(&python_string(&data), 200)
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_none_or(|number| number != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
    }
}

fn python_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => python_repr(value),
    }
}

fn python_repr(value: &Value) -> String {
    match value {
        Value::Null => "None".to_owned(),
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'")),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(python_repr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Object(values) => format!(
            "{{{}}}",
            values
                .iter()
                .map(|(key, value)| format!(
                    "'{}': {}",
                    key.replace('\'', "\\'"),
                    python_repr(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn take_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn clamp(value: i64, low: i64, high: i64) -> i64 {
    value.max(low).min(high)
}

fn quote_path(value: &str, additional_safe: &str) -> String {
    let mut output = String::new();
    for byte in value.trim().as_bytes() {
        let is_unreserved =
            byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_' | b'.' | b'~');
        if is_unreserved || additional_safe.as_bytes().contains(byte) {
            output.push(char::from(*byte));
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

fn paper_path(paper_id: &str) -> String {
    quote_path(paper_id, ":/")
}

fn compact(value: Value) -> Value {
    let Value::Object(mut object) = value else {
        return value;
    };
    object.retain(|_, value| !is_emptyish(value));
    Value::Object(object)
}

fn is_emptyish(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(value) => value.is_empty(),
        Value::Array(value) => value.is_empty(),
        Value::Object(value) => value.is_empty(),
        Value::Bool(_) | Value::Number(_) => false,
    }
}

fn truncate(value: Option<&str>, limit: Option<usize>) -> Option<String> {
    let value = value?;
    let Some(limit) = limit else {
        return Some(value.to_owned());
    };
    if value.chars().count() <= limit {
        return Some(value.to_owned());
    }
    Some(format!("{}{OMITTED}", take_chars(value, limit).trim_end()))
}

fn ids(external_ids: Option<&Value>) -> Value {
    let get = |key: &str| {
        external_ids
            .and_then(Value::as_object)
            .and_then(|ids| ids.get(key))
            .cloned()
            .unwrap_or(Value::Null)
    };
    compact(json!({
        "doi": get("DOI"),
        "arxiv": get("ArXiv"),
        "dblp": get("DBLP"),
        "corpusId": get("CorpusId"),
    }))
}

pub fn format_paper(raw: &Value, abstract_chars: Option<usize>) -> Value {
    let authors = raw
        .get("authors")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let shown_authors = authors
        .iter()
        .take(MAX_AUTHORS)
        .map(|author| {
            compact(json!({
                "name": author.get("name").cloned().unwrap_or(Value::Null),
                "authorId": author.get("authorId").cloned().unwrap_or(Value::Null),
            }))
        })
        .collect::<Vec<_>>();
    let pdf_url = raw
        .get("openAccessPdf")
        .and_then(Value::as_object)
        .and_then(|pdf| pdf.get("url"))
        .cloned()
        .unwrap_or(Value::Null);
    let tldr = raw
        .get("tldr")
        .and_then(Value::as_object)
        .and_then(|summary| summary.get("text"))
        .cloned()
        .unwrap_or(Value::Null);
    compact(json!({
        "paperId": raw.get("paperId").cloned().unwrap_or(Value::Null),
        "title": raw.get("title").cloned().unwrap_or(Value::Null),
        "year": raw.get("year").cloned().unwrap_or(Value::Null),
        "publicationDate": raw.get("publicationDate").cloned().unwrap_or(Value::Null),
        "venue": raw.get("venue").cloned().unwrap_or(Value::Null),
        "publicationTypes": raw.get("publicationTypes").cloned().unwrap_or(Value::Null),
        "authors": shown_authors,
        "authorCount": if authors.len() > MAX_AUTHORS { json!(authors.len()) } else { Value::Null },
        "citationCount": raw.get("citationCount").cloned().unwrap_or(Value::Null),
        "influentialCitationCount": raw.get("influentialCitationCount").cloned().unwrap_or(Value::Null),
        "referenceCount": raw.get("referenceCount").cloned().unwrap_or(Value::Null),
        "fieldsOfStudy": raw.get("fieldsOfStudy").cloned().unwrap_or(Value::Null),
        "ids": ids(raw.get("externalIds")),
        "url": raw.get("url").cloned().unwrap_or(Value::Null),
        "openAccessPdf": pdf_url,
        "tldr": tldr,
        "abstract": truncate(raw.get("abstract").and_then(Value::as_str), abstract_chars),
        "matchScore": raw.get("matchScore").cloned().unwrap_or(Value::Null),
    }))
}

pub fn format_link(item: &Value, paper_key: &str) -> Value {
    let paper = item.get(paper_key).cloned().unwrap_or_else(|| json!({}));
    let mut output = match format_paper(&paper, None) {
        Value::Object(object) => object,
        _ => Map::new(),
    };
    output.insert(
        "isInfluential".to_owned(),
        item.get("isInfluential").cloned().unwrap_or(Value::Null),
    );
    output.insert(
        "intents".to_owned(),
        item.get("intents").cloned().unwrap_or(Value::Null),
    );
    let contexts = item
        .get("contexts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(MAX_CONTEXTS)
        .filter_map(Value::as_str)
        .map(|context| truncate(Some(context), Some(CONTEXT_CHARS)).unwrap_or_default())
        .map(Value::String)
        .collect::<Vec<_>>();
    output.insert("contexts".to_owned(), Value::Array(contexts));
    compact(Value::Object(output))
}

fn match_not_found() -> Value {
    json!({"match": null, "note": "一致する論文が見つかりませんでした。"})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::VecDeque, sync::Mutex};

    #[derive(Clone, Debug)]
    struct CapturedRequest {
        url: String,
        headers: HashMap<String, String>,
        timeout: Duration,
    }

    struct FakeTransport {
        replies: Mutex<VecDeque<HttpResponse>>,
        calls: Mutex<Vec<CapturedRequest>>,
    }

    impl FakeTransport {
        fn new(replies: Vec<HttpResponse>) -> Self {
            Self {
                replies: Mutex::new(replies.into()),
                calls: Mutex::new(Vec::new()),
            }
        }

        fn calls(&self) -> Vec<CapturedRequest> {
            self.calls.lock().expect("calls mutex").clone()
        }
    }

    #[async_trait]
    impl HttpTransport for FakeTransport {
        async fn fetch(
            &self,
            url: &str,
            headers: &HashMap<String, String>,
            timeout: Duration,
        ) -> Result<HttpResponse, String> {
            self.calls
                .lock()
                .expect("calls mutex")
                .push(CapturedRequest {
                    url: url.to_owned(),
                    headers: headers.clone(),
                    timeout,
                });
            self.replies
                .lock()
                .expect("replies mutex")
                .pop_front()
                .ok_or_else(|| "no fake response".to_owned())
        }
    }

    #[derive(Default)]
    struct FakeClock {
        now: Mutex<f64>,
    }

    impl FakeClock {
        fn new(now: f64) -> Self {
            Self {
                now: Mutex::new(now),
            }
        }

        fn advance(&self, seconds: f64) {
            *self.now.lock().expect("clock mutex") += seconds;
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> f64 {
            *self.now.lock().expect("clock mutex")
        }
    }

    struct FakeSleeper {
        clock: Arc<FakeClock>,
        sleeps: Mutex<Vec<f64>>,
    }

    impl FakeSleeper {
        fn new(clock: Arc<FakeClock>) -> Self {
            Self {
                clock,
                sleeps: Mutex::new(Vec::new()),
            }
        }

        fn sleeps(&self) -> Vec<f64> {
            self.sleeps.lock().expect("sleeps mutex").clone()
        }
    }

    #[async_trait]
    impl Sleeper for FakeSleeper {
        async fn sleep(&self, seconds: f64) {
            self.sleeps.lock().expect("sleeps mutex").push(seconds);
            self.clock.advance(seconds);
        }
    }

    fn response(status: u16, body: Value, retry_after: Option<&str>) -> HttpResponse {
        HttpResponse {
            status,
            body: body.to_string(),
            retry_after: retry_after.map(ToOwned::to_owned),
        }
    }

    fn paper(number: usize, extra: Value) -> Value {
        let mut result = json!({
            "paperId": format!("p{number}"),
            "title": format!("Paper {number}"),
            "year": 2020,
            "authors": [{"name": "A", "authorId": "1"}],
            "externalIds": {"DOI": format!("10.1/{number}"), "CorpusId": number},
        });
        if let (Some(result), Value::Object(extra)) = (result.as_object_mut(), extra) {
            result.extend(extra);
        }
        result
    }

    fn install(
        replies: Vec<HttpResponse>,
        api_key: Option<&str>,
        interval: f64,
    ) -> (
        SemanticScholarClient,
        Arc<FakeTransport>,
        Arc<FakeClock>,
        Arc<FakeSleeper>,
    ) {
        let transport = Arc::new(FakeTransport::new(replies));
        let clock = Arc::new(FakeClock::new(100.0));
        let sleeper = Arc::new(FakeSleeper::new(Arc::clone(&clock)));
        let config = ClientConfig {
            api_key: api_key.map(ToOwned::to_owned),
            interval_seconds: interval,
            ..ClientConfig::default()
        };
        let client = SemanticScholarClient::with_dependencies(
            config,
            transport.clone(),
            clock.clone(),
            sleeper.clone(),
        );
        (client, transport, clock, sleeper)
    }

    #[tokio::test]
    async fn test_search_makes_one_request_and_clamps_limit() {
        let (client, fetch, _, _) = install(
            vec![response(
                200,
                json!({"total": 5000, "offset": 0, "next": 100, "data": [paper(1, json!({})), paper(2, json!({}))]}),
                None,
            )],
            None,
            1.1,
        );
        let result = client
            .search_papers(SearchPapersArgs {
                query: "raft consensus".to_owned(),
                limit: 500,
                offset: 0,
                year: None,
                venue: None,
                fields_of_study: None,
                min_citation_count: None,
            })
            .await;
        let calls = fetch.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].timeout, DEFAULT_TIMEOUT);
        assert!(
            calls[0]
                .url
                .starts_with(&format!("{API_BASE}/paper/search?"))
        );
        assert!(calls[0].url.contains("limit=100"));
        assert_eq!(result["total"], 5000);
        assert_eq!(result["papers"][0]["paperId"], "p1");
        assert_eq!(
            result["papers"][0]["ids"],
            json!({"doi":"10.1/1", "corpusId":1})
        );
        assert_eq!(calls[0].headers["User-Agent"], USER_AGENT);
    }

    #[tokio::test]
    async fn test_optional_filters_are_omitted_when_unset() {
        let (client, fetch, _, _) = install(
            vec![response(200, json!({"total": 0, "data": []}), None)],
            None,
            1.1,
        );
        client
            .search_papers(SearchPapersArgs {
                query: "paxos".to_owned(),
                limit: 10,
                offset: 0,
                year: None,
                venue: None,
                fields_of_study: None,
                min_citation_count: None,
            })
            .await;
        let url = &fetch.calls()[0].url;
        assert!(!url.contains("year="));
        assert!(!url.contains("venue="));
        assert!(!url.contains("minCitationCount="));
    }

    #[tokio::test]
    async fn test_api_key_header_only_when_set() {
        let (client, fetch, _, _) = install(
            vec![response(200, paper(1, json!({})), None)],
            Some("secret"),
            1.1,
        );
        client.get_paper("p1".to_owned()).await;
        assert_eq!(fetch.calls()[0].headers["x-api-key"], "secret");

        let (client, fetch, _, _) =
            install(vec![response(200, paper(1, json!({})), None)], None, 1.1);
        client.get_paper("p1".to_owned()).await;
        assert!(!fetch.calls()[0].headers.contains_key("x-api-key"));
    }

    #[test]
    fn test_from_env_reads_api_key() {
        let config = ClientConfig::from_values(Some("  k  "), Some("2.5")).expect("valid interval");
        assert_eq!(config.api_key.as_deref(), Some("k"));
        assert_eq!(config.interval_seconds, 2.5);
        let config = ClientConfig::from_values(Some("  "), None).expect("default interval");
        assert_eq!(config.api_key, None);
        assert_eq!(config.interval_seconds, DEFAULT_INTERVAL);
        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
    }

    #[tokio::test]
    async fn test_retries_429_honoring_retry_after() {
        let (client, fetch, _, sleeper) = install(
            vec![
                response(429, json!({"message": "Too Many Requests"}), Some("3")),
                response(200, paper(1, json!({})), None),
            ],
            None,
            1.1,
        );
        let result = client.get_paper("p1".to_owned()).await;
        assert_eq!(result["paperId"], "p1");
        assert_eq!(fetch.calls().len(), 2);
        assert!(sleeper.sleeps().contains(&3.0));
    }

    #[tokio::test]
    async fn test_gives_up_after_retries_with_key_hint() {
        let (client, fetch, _, sleeper) = install(
            vec![response(429, json!({"message": "Too Many Requests"}), None); 4],
            None,
            1.1,
        );
        let result = client.get_paper("p1".to_owned()).await;
        assert_eq!(fetch.calls().len(), 4);
        assert_eq!(result["status"], 429);
        assert!(
            result["hint"]
                .as_str()
                .unwrap_or_default()
                .contains("SEMANTIC_SCHOLAR_API_KEY")
        );
        assert_eq!(sleeper.sleeps(), vec![2.0, 4.0, 8.0]);
    }

    #[tokio::test]
    async fn test_rate_limiter_spaces_consecutive_requests() {
        let (client, _, _, sleeper) = install(
            vec![
                response(200, paper(1, json!({})), None),
                response(200, paper(2, json!({})), None),
            ],
            None,
            1.1,
        );
        client.get_paper("p1".to_owned()).await;
        client.get_paper("p2".to_owned()).await;
        let sleeps = sleeper.sleeps();
        assert_eq!(sleeps.len(), 1);
        assert!((sleeps[0] - 1.1).abs() < 1e-9);
    }

    #[tokio::test]
    async fn test_not_found_returns_hint() {
        let (client, _, _, _) = install(
            vec![response(
                404,
                json!({"error": "Paper with id x not found"}),
                None,
            )],
            None,
            1.1,
        );
        let result = client.get_paper("x".to_owned()).await;
        assert_eq!(result["status"], 404);
        assert!(
            result["error"]
                .as_str()
                .unwrap_or_default()
                .contains("not found")
        );
        assert!(result["hint"].as_str().unwrap_or_default().contains("DOI:"));
    }

    #[tokio::test]
    async fn test_match_paper_without_match() {
        let (client, _, _, _) = install(
            vec![response(
                404,
                json!({"error": "Title match not found"}),
                None,
            )],
            None,
            1.1,
        );
        let result = client
            .match_paper("A paper that does not exist".to_owned())
            .await;
        assert!(result["match"].is_null());
        assert_eq!(result["note"], "一致する論文が見つかりませんでした。");
    }

    #[tokio::test]
    async fn test_match_paper_returns_score() {
        let (client, _, _, _) = install(
            vec![response(
                200,
                json!({"data": [paper(1, json!({"matchScore": 212.3}))]}),
                None,
            )],
            None,
            1.1,
        );
        let result = client.match_paper("Paper 1".to_owned()).await;
        assert_eq!(result["match"]["matchScore"], 212.3);
    }

    #[test]
    fn test_format_paper_truncates_authors_and_abstract() {
        let mut raw = paper(1, json!({"abstract": "x".repeat(3000)}));
        raw["authors"] = Value::Array(
            (0..12)
                .map(|index| json!({"name": format!("Author {index}"), "authorId": index.to_string()}))
                .collect(),
        );
        let formatted = format_paper(&raw, Some(SEARCH_ABSTRACT_CHARS));
        assert_eq!(
            formatted["authors"].as_array().map(Vec::len),
            Some(MAX_AUTHORS)
        );
        assert_eq!(formatted["authorCount"], 12);
        assert!(
            formatted["abstract"]
                .as_str()
                .unwrap_or_default()
                .ends_with("（省略）")
        );
        assert!(
            formatted["abstract"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .count()
                < 1600
        );
    }

    #[test]
    fn test_format_paper_keeps_zero_and_false() {
        let linked = json!({
            "citingPaper": paper(1, json!({"citationCount": 0})),
            "isInfluential": false,
        });
        let formatted = format_link(&linked, "citingPaper");
        assert_eq!(formatted["citationCount"], 0);
        assert_eq!(formatted["isInfluential"], false);
    }

    #[tokio::test]
    async fn test_citations_use_citing_paper_and_cap_contexts() {
        let item = json!({
            "citingPaper": paper(7, json!({})),
            "contexts": ["c".repeat(500), "c".repeat(500), "c".repeat(500), "c".repeat(500), "c".repeat(500)],
            "intents": ["background"],
            "isInfluential": true,
        });
        let (client, fetch, _, _) = install(
            vec![response(
                200,
                json!({"offset": 0, "next": 20, "data": [item]}),
                None,
            )],
            None,
            1.1,
        );
        let result = client.get_citations("p1".to_owned(), 1000, 0).await;
        let url = &fetch.calls()[0].url;
        assert!(url.contains("/paper/p1/citations?"));
        assert!(url.contains("limit=100"));
        let citing = &result["citations"][0];
        assert_eq!(citing["paperId"], "p7");
        assert_eq!(
            citing["contexts"].as_array().map(Vec::len),
            Some(MAX_CONTEXTS)
        );
        assert!(
            citing["contexts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|context| context.as_str().unwrap().chars().count() < 320)
        );
    }

    #[tokio::test]
    async fn test_references_use_cited_paper() {
        let (client, _, _, _) = install(
            vec![response(
                200,
                json!({"offset": 0, "data": [{"citedPaper": paper(3, json!({}))}]}),
                None,
            )],
            None,
            1.1,
        );
        let result = client.get_references("p1".to_owned(), 50, 0).await;
        assert_eq!(result["references"][0]["paperId"], "p3");
    }

    #[tokio::test]
    async fn test_paper_id_keeps_prefix_and_quotes_spaces() {
        let (client, fetch, _, _) = install(
            vec![
                response(200, paper(1, json!({})), None),
                response(200, paper(2, json!({})), None),
            ],
            None,
            0.0,
        );
        client
            .get_paper("DOI:10.1145/3132747.3132757".to_owned())
            .await;
        client.get_paper("bad id".to_owned()).await;
        let calls = fetch.calls();
        assert!(calls[0].url.contains("/paper/DOI:10.1145/3132747.3132757?"));
        assert!(calls[1].url.contains("/paper/bad%20id?"));
    }

    #[tokio::test]
    async fn test_get_author_fetches_papers() {
        let author = json!({"authorId": "1741101", "name": "Leslie Lamport", "hIndex": 80});
        let (client, fetch, _, _) = install(
            vec![
                response(200, author, None),
                response(200, json!({"data": [paper(1, json!({}))]}), None),
            ],
            None,
            1.1,
        );
        let result = client.get_author("1741101".to_owned(), 5).await;
        assert_eq!(result["name"], "Leslie Lamport");
        assert_eq!(result["papers"][0]["paperId"], "p1");
        assert!(fetch.calls()[1].url.contains("limit=5"));
    }

    #[tokio::test]
    async fn test_get_author_skips_papers_when_zero() {
        let (client, fetch, _, _) = install(
            vec![response(200, json!({"authorId": "1", "name": "A"}), None)],
            None,
            1.1,
        );
        let result = client.get_author("1".to_owned(), 0).await;
        assert_eq!(fetch.calls().len(), 1);
        assert!(result.get("papers").is_none());
    }
}
