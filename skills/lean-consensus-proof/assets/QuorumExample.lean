module

public import Mathlib

/-!
# quorum の交差と、1回だけ投票するプロトコルの safety

分散合意の safety の証明で繰り返し使う quorum の交差の補題と、それを使った小さな
プロトコルの証明の例。ノードの集合を `Finset` で表し、quorum の大きさの条件から交差を導く。

* `Quorum.majority_quorums_intersect`: 過半数の quorum どうしは交わる
* `Quorum.bft_quorums_share_correct`: `n = 3f+1` で大きさ `2f+1` の quorum どうしは正しいノードを共有する
* `Quorum.decided_values_agree`: 各ノードが1回しか投票しなければ、過半数で決まった値は1つ
* `Voting.agreement`、`Voting.decided_stable`: 上を状態機械として書いた例

Lean v4.35.0、Mathlib v4.35.0 で確認（2026-10-31）。
-/

public section

namespace Quorum

variable {α : Type*}

/-- 全体 `U` の過半数を占める2つの quorum は、少なくとも1つのノードを共有する。 -/
theorem majority_quorums_intersect [DecidableEq α] (U Q₁ Q₂ : Finset α)
    (h₁ : Q₁ ⊆ U) (h₂ : Q₂ ⊆ U)
    (hq₁ : U.card < 2 * Q₁.card) (hq₂ : U.card < 2 * Q₂.card) :
    (Q₁ ∩ Q₂).Nonempty := by
  rw [Finset.nonempty_iff_ne_empty]
  intro h
  have key := Finset.card_union_add_card_inter Q₁ Q₂
  rw [h, Finset.card_empty] at key
  have hsub := Finset.card_le_card (Finset.union_subset h₁ h₂)
  omega

/-- `n = 3f+1` のとき、大きさ `2f+1` 以上の2つの quorum は、故障したノードの集合 `F`
（大きさ `f` 以下）に入らないノード、つまり正しいノードを少なくとも1つ共有する。 -/
theorem bft_quorums_share_correct [DecidableEq α] (U Q₁ Q₂ F : Finset α) (f : ℕ)
    (h₁ : Q₁ ⊆ U) (h₂ : Q₂ ⊆ U) (hU : U.card = 3 * f + 1)
    (hq₁ : 2 * f + 1 ≤ Q₁.card) (hq₂ : 2 * f + 1 ≤ Q₂.card) (hF : F.card ≤ f) :
    ∃ x ∈ Q₁ ∩ Q₂, x ∉ F := by
  have key := Finset.card_union_add_card_inter Q₁ Q₂
  have hsub := Finset.card_le_card (Finset.union_subset h₁ h₂)
  by_contra hcon
  push Not at hcon
  have hle := Finset.card_le_card (show Q₁ ∩ Q₂ ⊆ F from fun x hx => hcon x hx)
  omega

/-- 各ノードが高々1つの値にしか投票しないこと。 -/
@[expose] def VotesOnce {V : Type*} (vote : α → V → Prop) : Prop :=
  ∀ a v w, vote a v → vote a w → v = w

/-- 各ノードが1回しか投票しないとき、過半数の quorum の全員が投票した値は1つに決まる。
単発の合意（single-decree）の safety の核にあたる。 -/
theorem decided_values_agree {V : Type*} (U : Finset α) (vote : α → V → Prop)
    (hOnce : VotesOnce vote) (v w : V) (Q₁ Q₂ : Finset α)
    (h₁ : Q₁ ⊆ U) (h₂ : Q₂ ⊆ U)
    (hq₁ : U.card < 2 * Q₁.card) (hq₂ : U.card < 2 * Q₂.card)
    (hv : ∀ a ∈ Q₁, vote a v) (hw : ∀ a ∈ Q₂, vote a w) : v = w := by
  classical
  obtain ⟨a, ha⟩ := majority_quorums_intersect U Q₁ Q₂ h₁ h₂ hq₁ hq₂
  rw [Finset.mem_inter] at ha
  exact hOnce a v w (hv a ha.1) (hw a ha.2)

end Quorum

namespace Voting

variable {α V : Type*}

/-- 状態: 各ノードの投票。`none` はまだ投票していないこと。 -/
structure State (α V : Type*) where
  votes : α → Option V

/-- 初期状態: 誰も投票していない。 -/
def Init (s : State α V) : Prop :=
  ∀ a, s.votes a = none

/-- 遷移: まだ投票していないノード `a` が、値 `v` に投票する。 -/
def Next [DecidableEq α] (s s' : State α V) : Prop :=
  ∃ a v, s.votes a = none ∧ s'.votes = Function.update s.votes a (some v)

/-- 値 `v` が決まった: 過半数の quorum の全員が `v` に投票している。 -/
def Decided (U : Finset α) (s : State α V) (v : V) : Prop :=
  ∃ Q ⊆ U, U.card < 2 * Q.card ∧ ∀ a ∈ Q, s.votes a = some v

/-- agreement: どの状態でも、決まった2つの値は等しい。 -/
theorem agreement (U : Finset α) (s : State α V) (v w : V)
    (hv : Decided U s v) (hw : Decided U s w) : v = w := by
  obtain ⟨Q₁, h₁, hq₁, hall₁⟩ := hv
  obtain ⟨Q₂, h₂, hq₂, hall₂⟩ := hw
  have hOnce : Quorum.VotesOnce (fun a x => s.votes a = some x) := by
    unfold Quorum.VotesOnce
    intro a x y hx hy
    rw [hx] at hy
    exact Option.some.inj hy
  exact Quorum.decided_values_agree U (fun a x => s.votes a = some x) hOnce v w Q₁ Q₂
    h₁ h₂ hq₁ hq₂ hall₁ hall₂

/-- 決まった値は、遷移のあとも決まったまま。 -/
theorem decided_stable [DecidableEq α] (U : Finset α) (s s' : State α V) (v : V)
    (hn : Next s s') (hd : Decided U s v) : Decided U s' v := by
  obtain ⟨b, x, hb, hs'⟩ := hn
  obtain ⟨Q, hQU, hQ, hall⟩ := hd
  refine ⟨Q, hQU, hQ, fun a ha => ?_⟩
  have hab : a ≠ b := by
    rintro rfl
    rw [hall a ha] at hb
    exact Option.some_ne_none v hb
  rw [hs', Function.update_of_ne hab]
  exact hall a ha

end Voting

end
