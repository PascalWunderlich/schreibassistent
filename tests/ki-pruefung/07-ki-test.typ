// Testkapitel für P8 (KI-Prüfungen). Die Kommentare TESTFALL gehen nicht an die KI.
// 3 unbelegte Aussagen (U1–U3), 2 falsch belegte (F1, F2); alle anderen Sätze sind
// richtig belegt oder eigene Überlegungen ohne Belegbedarf.

= KI-Testkapitel <kap:ki-test>

== Ausgangslage

In diesem Kapitel wird untersucht, unter welchen Bedingungen sich die Ablösung eines Monolithen lohnt. Ein Monolith ist eine Anwendung, deren Module nicht unabhängig voneinander ausgeführt werden können @dragoni2016[S. 1]. Laut einer Umfrage aus dem Jahr 2020 setzen bereits 63 % der Unternehmen Microservices produktiv ein. // TESTFALL U1 (Zahl ohne Beleg)

Der Begriff „Microservices“ wurde erstmals 2011 auf einem Workshop von Softwarearchitekten verwendet. // TESTFALL U2 (Tatsache ohne Beleg)
Als wichtigste Gründe für eine Migration nennen Unternehmen Wartbarkeit und Skalierbarkeit @fritzsch2019[S. 1]. Die meisten Unternehmen zerlegen dabei ihre bestehende Codebasis schrittweise, statt die Anwendung neu zu entwickeln @fritzsch2019[S. 1]. // TESTFALL F1 (Quelle sagt das Gegenteil: Neuentwicklung bevorzugt)

== Lernverfahren für den Betrieb

Für die automatische Steuerung von Diensten bietet sich bestärkendes Lernen an. Beim Experience Replay werden frühere Übergänge zufällig erneut zum Lernen verwendet, wodurch Korrelationen zwischen aufeinanderfolgenden Zuständen aufgebrochen werden. // TESTFALL U3 (Definition ohne Beleg)
Das DQN-Verfahren benötigt ein Modell der Umgebung, um seine Strategie zu lernen @mnih2013[S. 3]. // TESTFALL F2 (Quelle: modellfrei)

Im Folgenden werden beide Ansätze für das Praxisbeispiel verglichen.
