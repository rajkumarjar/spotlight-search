import { useCallback, useEffect, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

const WINDOW_WIDTH = 760;
const WINDOW_PADDING = 80;

export function usePanelAnimation() {
  const contentRef = useRef<HTMLDivElement>(null);
  const [isVisible, setIsVisible] = useState(false);
  const [panelHeight, setPanelHeight] = useState<number>();

  const openPanel = useCallback(() => {
    setIsVisible(true);
  }, []);

  const closePanel = useCallback(() => {
    setIsVisible(false);
    if (!isTauri()) return;

    void invoke("hide_window").catch((error: unknown) => {
      console.error("Unable to hide the Spotlight demo window.", error);
    });
  }, []);

  const markPanelHidden = useCallback(() => {
    setIsVisible(false);
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      setIsVisible(true);
      return;
    }

    let active = true;
    void getCurrentWindow()
      .isVisible()
      .then((visible) => {
        if (active && visible) setIsVisible(true);
      })
      .catch((error: unknown) => {
        console.error("Unable to read Spotlight Search visibility.", error);
      });

    return () => {
      active = false;
    };
  }, []);

  const resizePanel = useCallback(() => {
    const content = contentRef.current;
    if (!content) return;

    const contentHeight = Math.ceil(content.getBoundingClientRect().height);
    setPanelHeight(contentHeight + 2);
    if (!isTauri()) return;

    const height = contentHeight + 2 + WINDOW_PADDING;
    void getCurrentWindow()
      .setSize(new LogicalSize(WINDOW_WIDTH, height))
      .catch((error: unknown) => {
        console.error("Unable to resize the Spotlight demo window.", error);
      });
  }, []);

  useEffect(() => {
    const content = contentRef.current;
    if (!content) return;

    let frame = 0;
    const observer = new ResizeObserver(() => {
      window.cancelAnimationFrame(frame);
      frame = window.requestAnimationFrame(resizePanel);
    });
    observer.observe(content);

    return () => {
      observer.disconnect();
      window.cancelAnimationFrame(frame);
    };
  }, [resizePanel]);

  return { contentRef, isVisible, panelHeight, openPanel, closePanel, markPanelHidden };
}
