// Test-Vault P5: zitierte Quelle, die es in der .bib nicht gibt. Typst setzt das nicht
// („label does not exist“); die Prüfung meldet es trotzdem aus dem Quelltext.
= Einleitung

// FEHLERFALL quellen (Citekey fehlt in der .bib)
Wie gezeigt @gibtsnicht2020, siehe auch @vorhanden2020.

#bibliography("../System/quellen.bib")
