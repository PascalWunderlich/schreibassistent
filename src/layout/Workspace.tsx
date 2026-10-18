import { ReactNode, useEffect } from "react";
import { PANEL_EVENT, PanelSide } from "../lib/events";
import { Group, Panel, Separator, useDefaultLayout, usePanelRef } from "react-resizable-panels";

interface Props {
  left: ReactNode;
  center: ReactNode;
  right: ReactNode;
}

const PANELS = ["left", "center", "right"];

/** Drei Spalten: Kapitel | Blatt | Prüfen und Assistent. Breiten werden gemerkt, Menü Darstellung blendet die Seiten ein und aus (⌘⌥←, ⌘⌥→). */
export function Workspace({ left, center, right }: Props) {
  const leftRef = usePanelRef();
  const rightRef = usePanelRef();
  const layout = useDefaultLayout({ id: "workspace", panelIds: PANELS, storage: localStorage });

  useEffect(() => {
    const toggle = (ref: typeof leftRef) => {
      const panel = ref.current;
      if (!panel) return;
      if (panel.isCollapsed()) panel.expand();
      else panel.collapse();
    };
    // Die Tastenkürzel ⌘⌥← und ⌘⌥→ kommen über das Menü „Darstellung“ als Ereignis.
    const onPanel = (e: Event) => toggle((e as CustomEvent<PanelSide>).detail === "links" ? leftRef : rightRef);
    window.addEventListener(PANEL_EVENT, onPanel);
    return () => window.removeEventListener(PANEL_EVENT, onPanel);
  }, [leftRef, rightRef]);

  const handle = "w-px bg-border transition-colors hover:bg-ring data-[separator=active]:bg-ring focus-visible:bg-ring outline-none";

  return (
    <Group
      id="workspace"
      orientation="horizontal"
      className="flex-1 min-h-0"
      defaultLayout={layout.defaultLayout}
      onLayoutChanged={layout.onLayoutChanged}
    >
      <Panel id="left" panelRef={leftRef} defaultSize={240} minSize={200} maxSize={420} collapsible collapsedSize={0}>
        {left}
      </Panel>
      <Separator className={handle} />
      <Panel id="center" minSize={360}>
        {center}
      </Panel>
      <Separator className={handle} />
      <Panel id="right" panelRef={rightRef} defaultSize={320} minSize={260} maxSize={560} collapsible collapsedSize={0}>
        {right}
      </Panel>
    </Group>
  );
}
