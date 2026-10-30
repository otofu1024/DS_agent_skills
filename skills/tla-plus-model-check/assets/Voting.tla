---------------------------- MODULE Voting ----------------------------
(***************************************************************************)
(* 各ノードが値に1回だけ投票し、過半数が投票した値を「決まった値」として  *)
(* chosen に記録するプロトコル。Agreement は「決まった値は1つまで」。     *)
(*                                                                         *)
(* Spec は正しい規則（まだ投票していないノードだけが投票する）。           *)
(* SpecBuggy は誤った規則（投票済みのノードも投票し直せる）で、TLC が       *)
(* Agreement の反例を見つける。                                            *)
(***************************************************************************)
EXTENDS Naturals, FiniteSets

CONSTANTS Node, Value, None

VARIABLES votes, chosen

vars == <<votes, chosen>>

TypeOK ==
    /\ votes \in [Node -> Value \cup {None}]
    /\ chosen \subseteq Value

IsQuorum(Q) == 2 * Cardinality(Q) > Cardinality(Node)

Decided(v) == \E Q \in SUBSET Node : IsQuorum(Q) /\ \A n \in Q : votes[n] = v

Init ==
    /\ votes = [n \in Node |-> None]
    /\ chosen = {}

\* 正しい規則: まだ投票していないノードだけが投票する
Vote(n, v) ==
    /\ votes[n] = None
    /\ votes' = [votes EXCEPT ![n] = v]
    /\ UNCHANGED chosen

\* 誤った規則: 投票済みのノードも投票し直せる
Revote(n, v) ==
    /\ votes' = [votes EXCEPT ![n] = v]
    /\ UNCHANGED chosen

\* 過半数が投票した値を、決まった値として記録する
Learn(v) ==
    /\ Decided(v)
    /\ chosen' = chosen \cup {v}
    /\ UNCHANGED votes

Next ==
    \/ \E n \in Node, v \in Value : Vote(n, v)
    \/ \E v \in Value : Learn(v)

NextBuggy ==
    \/ \E n \in Node, v \in Value : Revote(n, v)
    \/ \E v \in Value : Learn(v)

Spec == Init /\ [][Next]_vars
SpecBuggy == Init /\ [][NextBuggy]_vars

Agreement == Cardinality(chosen) <= 1
=============================================================================
