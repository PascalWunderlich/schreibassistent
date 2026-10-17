import { useState } from "react";
import { PageEditor } from "./prototypes/a/PageEditor";
import { PreviewEditor } from "./prototypes/b/PreviewEditor";
import "./App.css";

type Prototype = "a" | "b";

function App() {
  const [prototype, setPrototype] = useState<Prototype>("a");

  return (
    <div className="app">
      <header className="topbar">
        <h1>Schreibassistent</h1>
        <nav>
          <button aria-pressed={prototype === "a"} onClick={() => setPrototype("a")}>
            A: Seiteneditor
          </button>
          <button aria-pressed={prototype === "b"} onClick={() => setPrototype("b")}>
            B: Vorschau
          </button>
        </nav>
      </header>
      <main>{prototype === "a" ? <PageEditor /> : <PreviewEditor />}</main>
    </div>
  );
}

export default App;
