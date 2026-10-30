use rmcp::{
    Json, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{Implementation, ServerCapabilities, ServerConfig},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{SearchPapersArgs, SemanticScholarClient};

pub const SERVER_INSTRUCTIONS: &str = "Semantic Scholar で論文を検索し、書誌情報・要旨・引用関係を返す。リクエストの間隔はサーバーが守るので、呼び出しを並列にしたり自分で待ったりしなくてよい。論文 ID には S2 の paperId のほか、DOI:...、ARXIV:...、CorpusId:... を使える。";

fn tool_output(value: Value) -> Json<Map<String, Value>> {
    match value {
        Value::Object(object) => Json(object),
        _ => Json(Map::new()),
    }
}

fn search_offset_default() -> i64 {
    0
}

fn citations_limit_default() -> i64 {
    20
}

fn references_limit_default() -> i64 {
    50
}

fn author_paper_limit_default() -> i64 {
    10
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct MatchPaperArgs {
    #[schemars(description = "論文の題名（完全でなくてよい）。")]
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetPaperArgs {
    #[schemars(
        description = "S2 の paperId、または DOI:...、ARXIV:...、CorpusId:... の形式の ID。"
    )]
    pub paper_id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetCitationsArgs {
    #[schemars(
        description = "S2 の paperId、または DOI:...、ARXIV:...、CorpusId:... の形式の ID。"
    )]
    pub paper_id: String,
    #[serde(default = "citations_limit_default")]
    #[schemars(
        default = "citations_limit_default",
        description = "返す件数（1〜100、既定 20）。"
    )]
    pub limit: i64,
    #[serde(default = "search_offset_default")]
    #[schemars(
        default = "search_offset_default",
        description = "何件目から返すか（ページ送り用）。"
    )]
    pub offset: i64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetReferencesArgs {
    #[schemars(
        description = "S2 の paperId、または DOI:...、ARXIV:...、CorpusId:... の形式の ID。"
    )]
    pub paper_id: String,
    #[serde(default = "references_limit_default")]
    #[schemars(
        default = "references_limit_default",
        description = "返す件数（1〜100、既定 50）。"
    )]
    pub limit: i64,
    #[serde(default = "search_offset_default")]
    #[schemars(
        default = "search_offset_default",
        description = "何件目から返すか（ページ送り用）。"
    )]
    pub offset: i64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetAuthorArgs {
    #[schemars(description = "S2 の authorId（論文の authors に含まれる）。")]
    pub author_id: String,
    #[serde(default = "author_paper_limit_default")]
    #[schemars(
        default = "author_paper_limit_default",
        description = "一緒に返す論文の数（0〜100、既定 10）。並びは API が返す順。"
    )]
    pub paper_limit: i64,
}

#[derive(Clone)]
pub struct SemanticScholarServer {
    pub tool_router: ToolRouter<Self>,
    client: SemanticScholarClient,
}

impl SemanticScholarServer {
    pub fn new(client: SemanticScholarClient) -> Self {
        Self {
            tool_router: Self::tool_router(),
            client,
        }
    }
}

#[tool_router]
impl SemanticScholarServer {
    #[tool(
        name = "search_papers",
        description = "キーワードで論文を関連度順に検索する。1回の呼び出しで API へのリクエストは1回だけ。\n\nArgs:\n    query: 検索語。専門用語を使った英語の表現が最もよく当たる。\n    limit: 返す件数（1〜100、既定 10）。\n    offset: 何件目から返すか（ページ送り用）。\n    year: 出版年の絞り込み。\"2019\"、\"2016-2020\"、\"2020-\"、\"-2010\" の形式。\n    venue: 会議・論文誌名の絞り込み（カンマ区切り）。表記揺れが多いので、外して検索してから要旨で選ぶ方が確実。\n    fields_of_study: 分野の絞り込み（例: \"Computer Science\"）。\n    min_citation_count: 被引用数の下限。"
    )]
    async fn search_papers(
        &self,
        Parameters(args): Parameters<SearchPapersArgs>,
    ) -> Json<Map<String, Value>> {
        tool_output(self.client.search_papers(args).await)
    }

    #[tool(
        name = "match_paper",
        description = "題名に最もよく一致する論文を1件返す。記憶にある論文が実在するかを確かめるときに使う。\n\nArgs:\n    title: 論文の題名（完全でなくてよい）。"
    )]
    async fn match_paper(
        &self,
        Parameters(args): Parameters<MatchPaperArgs>,
    ) -> Json<Map<String, Value>> {
        tool_output(self.client.match_paper(args.title).await)
    }

    #[tool(
        name = "get_paper",
        description = "1本の論文の書誌情報・要旨・TLDR・被引用数などを返す。\n\nArgs:\n    paper_id: S2 の paperId、または DOI:...、ARXIV:...、CorpusId:... の形式の ID。"
    )]
    async fn get_paper(
        &self,
        Parameters(args): Parameters<GetPaperArgs>,
    ) -> Json<Map<String, Value>> {
        tool_output(self.client.get_paper(args.paper_id).await)
    }

    #[tool(
        name = "get_citations",
        description = "その論文を引用している論文（後続研究）の一覧を返す。\n\n並びは API が返す順で、被引用数の順ではない。重要な後続研究を探すときは\nisInfluential、intents、citationCount を見て選ぶ。\n\nArgs:\n    paper_id: S2 の paperId、または DOI:...、ARXIV:...、CorpusId:... の形式の ID。\n    limit: 返す件数（1〜100、既定 20）。\n    offset: 何件目から返すか（ページ送り用）。"
    )]
    async fn get_citations(
        &self,
        Parameters(args): Parameters<GetCitationsArgs>,
    ) -> Json<Map<String, Value>> {
        tool_output(
            self.client
                .get_citations(args.paper_id, args.limit, args.offset)
                .await,
        )
    }

    #[tool(
        name = "get_references",
        description = "その論文が引用している論文（先行研究）の一覧を返す。\n\nArgs:\n    paper_id: S2 の paperId、または DOI:...、ARXIV:...、CorpusId:... の形式の ID。\n    limit: 返す件数（1〜100、既定 50）。\n    offset: 何件目から返すか（ページ送り用）。"
    )]
    async fn get_references(
        &self,
        Parameters(args): Parameters<GetReferencesArgs>,
    ) -> Json<Map<String, Value>> {
        tool_output(
            self.client
                .get_references(args.paper_id, args.limit, args.offset)
                .await,
        )
    }

    #[tool(
        name = "get_author",
        description = "著者の情報（所属、論文数、被引用数、h 指数）と論文の一部を返す。\n\nArgs:\n    author_id: S2 の authorId（論文の authors に含まれる）。\n    paper_limit: 一緒に返す論文の数（0〜100、既定 10）。並びは API が返す順。"
    )]
    async fn get_author(
        &self,
        Parameters(args): Parameters<GetAuthorArgs>,
    ) -> Json<Map<String, Value>> {
        tool_output(
            self.client
                .get_author(args.author_id, args.paper_limit)
                .await,
        )
    }
}

#[tool_handler]
impl ServerHandler for SemanticScholarServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(SERVER_INSTRUCTIONS)
            .with_server_info(Implementation::new("semantic-scholar", "0.1.0"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClientConfig, Clock, HttpResponse, HttpTransport, SemanticScholarClient, Sleeper};
    use async_trait::async_trait;
    use std::{collections::HashMap, sync::Arc, time::Duration};

    struct EmptyTransport;

    #[async_trait]
    impl HttpTransport for EmptyTransport {
        async fn fetch(
            &self,
            _url: &str,
            _headers: &HashMap<String, String>,
            _timeout: Duration,
        ) -> Result<HttpResponse, String> {
            Err("unused".to_owned())
        }
    }

    struct TestClock;

    impl Clock for TestClock {
        fn now(&self) -> f64 {
            0.0
        }
    }

    struct TestSleeper;

    #[async_trait]
    impl Sleeper for TestSleeper {
        async fn sleep(&self, _seconds: f64) {}
    }

    fn test_server() -> SemanticScholarServer {
        let client = SemanticScholarClient::with_dependencies(
            ClientConfig::default(),
            Arc::new(EmptyTransport),
            Arc::new(TestClock),
            Arc::new(TestSleeper),
        );
        SemanticScholarServer::new(client)
    }

    #[test]
    fn test_registered_tools() {
        let mut names = SemanticScholarServer::tool_router()
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            [
                "get_author",
                "get_citations",
                "get_paper",
                "get_references",
                "match_paper",
                "search_papers",
            ]
        );
    }

    #[test]
    fn test_server_advertises_exactly_six_tools() {
        let server = test_server();
        let tools = server.tool_router.list_all();
        let info = ServerHandler::get_info(&server);
        assert_eq!(tools.len(), 6);
        assert!(info.capabilities.tools.is_some());
        assert_eq!(info.instructions.as_deref(), Some(SERVER_INSTRUCTIONS));
        let mut names = tools
            .iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            [
                "get_author",
                "get_citations",
                "get_paper",
                "get_references",
                "match_paper",
                "search_papers",
            ]
        );

        let search = tools
            .iter()
            .find(|tool| tool.name == "search_papers")
            .expect("search tool is registered");
        assert_eq!(
            search.description.as_deref(),
            Some(
                "キーワードで論文を関連度順に検索する。1回の呼び出しで API へのリクエストは1回だけ。\n\nArgs:\n    query: 検索語。専門用語を使った英語の表現が最もよく当たる。\n    limit: 返す件数（1〜100、既定 10）。\n    offset: 何件目から返すか（ページ送り用）。\n    year: 出版年の絞り込み。\"2019\"、\"2016-2020\"、\"2020-\"、\"-2010\" の形式。\n    venue: 会議・論文誌名の絞り込み（カンマ区切り）。表記揺れが多いので、外して検索してから要旨で選ぶ方が確実。\n    fields_of_study: 分野の絞り込み（例: \"Computer Science\"）。\n    min_citation_count: 被引用数の下限。"
            )
        );
        let properties = search
            .input_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("search input schema has properties");
        assert_eq!(properties["limit"]["default"], serde_json::json!(10));
        assert_eq!(properties["offset"]["default"], serde_json::json!(0));
        let output_schema = search.output_schema.as_ref().expect("output schema exists");
        assert_eq!(
            output_schema.get("type"),
            Some(&Value::String("object".to_owned()))
        );
    }
}
