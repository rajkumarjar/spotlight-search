import type { LucideIcon } from "lucide-react";

export interface SearchItem {
  id: string;
  name: string;
  kind: string;
  detail: string;
  Icon: LucideIcon;
  tint: "blue" | "cyan" | "red" | "slate" | "pink";
}

export interface SearchHit {
  id: string;
  name: string;
  subtitle: string;
  kind: string;
}

export interface SearchReply {
  results: SearchHit[];
  warning: string | null;
}
