import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { IconApps } from "@tabler/icons-react";
import { listSessionExtensionStatus } from "@/features/extensions/api/extensions";
import {
  getDisplayName,
  type SessionExtensionStatus,
} from "@/features/extensions/types";
import { cn } from "@/shared/lib/cn";
import type { Message, ToolRequestContent } from "@/shared/types/messages";
import { useChatStore } from "../../stores/chatStore";
import { Widget } from "./Widget";

interface ExtensionsWidgetProps {
  sessionId: string;
}

interface ExtensionUsage {
  count: number;
  lastUsedAt: number;
}

const EMPTY_MESSAGES: Message[] = [];

function normalizeName(name: string): string {
  return name
    .replace(/\s/g, "")
    .replace(/[^a-zA-Z0-9_-]/g, "_")
    .toLowerCase();
}

function toolOwnerFromName(name: string): string | null {
  const [owner] = name.split("__");
  return owner && owner !== name ? normalizeName(owner) : null;
}

function getToolOwnerFromName(
  toolName: string,
  toolToExtension: Map<string, string>,
): string | null {
  return (
    toolToExtension.get(normalizeName(toolName)) ?? toolOwnerFromName(toolName)
  );
}

function getToolOwner(
  toolRequest: ToolRequestContent,
  toolToExtension: Map<string, string>,
): string | null {
  if (toolRequest.extensionName) {
    return normalizeName(toolRequest.extensionName);
  }
  if (toolRequest.toolName) {
    return getToolOwnerFromName(toolRequest.toolName, toolToExtension);
  }
  return getToolOwnerFromName(toolRequest.name, toolToExtension);
}

function ExtensionRow({ extension }: { extension: SessionExtensionStatus }) {
  const { t } = useTranslation("chat");
  const displayName = getDisplayName(extension);
  const isConnected = extension.status === "connected";
  const toolCount = extension.tools.length;

  return (
    <div className="flex min-w-0 items-start gap-2" title={extension.error}>
      <span
        className={cn(
          "mt-1.5 size-1.5 shrink-0 rounded-full",
          isConnected ? "bg-green-500" : "bg-amber-500",
        )}
      />
      <div className="min-w-0 flex-1">
        <div className="truncate text-xs text-foreground">{displayName}</div>
        <div
          className={cn(
            "mt-0.5 truncate text-[11px]",
            isConnected ? "text-foreground-subtle" : "text-amber-600",
          )}
        >
          {isConnected
            ? t("contextPanel.widgets.statusConnected")
            : t("contextPanel.widgets.statusFailed")}
          {isConnected && toolCount > 0
            ? ` · ${t("contextPanel.widgets.toolCount", { count: toolCount })}`
            : null}
        </div>
      </div>
    </div>
  );
}

export function ExtensionsWidget({ sessionId }: ExtensionsWidgetProps) {
  const { t } = useTranslation("chat");
  const [extensions, setExtensions] = useState<SessionExtensionStatus[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const messages = useChatStore(
    (s) => s.messagesBySession[sessionId] ?? EMPTY_MESSAGES,
  );

  const fetchStatuses = useCallback(() => {
    setIsLoading(true);
    listSessionExtensionStatus(sessionId)
      .then((all) => setExtensions(all))
      .catch(() => setExtensions([]))
      .finally(() => setIsLoading(false));
  }, [sessionId]);

  const toolRequestSignature = useMemo(() => {
    return messages
      .flatMap((message) =>
        message.content
          .filter((content) => content.type === "toolRequest")
          .map(
            (content) =>
              `${content.id}:${content.name}:${content.toolName ?? ""}:${content.extensionName ?? ""}:${content.status}`,
          ),
      )
      .join("|");
  }, [messages]);

  useEffect(() => {
    if (!toolRequestSignature) {
      setExtensions([]);
      setIsLoading(false);
      return;
    }
    fetchStatuses();
  }, [fetchStatuses, toolRequestSignature]);

  const toolToExtension = useMemo(() => {
    const byTool = new Map<string, string>();
    for (const extension of extensions) {
      for (const tool of extension.tools) {
        byTool.set(normalizeName(tool), extension.config_key);
        const unprefixedName = tool.split("__").pop();
        const unprefixedKey = unprefixedName
          ? normalizeName(unprefixedName)
          : null;
        if (unprefixedKey && !byTool.has(unprefixedKey)) {
          byTool.set(unprefixedKey, extension.config_key);
        }
      }
    }
    return byTool;
  }, [extensions]);

  const usageByExtension = useMemo(() => {
    const usage = new Map<string, ExtensionUsage>();
    for (const message of messages) {
      for (const content of message.content) {
        if (content.type !== "toolRequest") continue;
        const owner = getToolOwner(content, toolToExtension);
        if (!owner) continue;
        const previous = usage.get(owner);
        usage.set(owner, {
          count: (previous?.count ?? 0) + 1,
          lastUsedAt: Math.max(previous?.lastUsedAt ?? 0, message.created),
        });
      }
    }
    return usage;
  }, [messages, toolToExtension]);

  const used = useMemo(
    () =>
      extensions
        .filter((ext) => usageByExtension.has(ext.config_key))
        .sort((a, b) => {
          const aUsage = usageByExtension.get(a.config_key)?.lastUsedAt ?? 0;
          const bUsage = usageByExtension.get(b.config_key)?.lastUsedAt ?? 0;
          return bUsage - aUsage;
        }),
    [extensions, usageByExtension],
  );

  const renderSection = (sectionExtensions: SessionExtensionStatus[]) => {
    if (sectionExtensions.length === 0) return null;
    return (
      <div className="space-y-2">
        {sectionExtensions.map((ext) => (
          <ExtensionRow key={ext.config_key} extension={ext} />
        ))}
      </div>
    );
  };

  return (
    <Widget
      title={t("contextPanel.widgets.extensionsUsedTitle")}
      icon={<IconApps className="size-3.5" />}
      flush
    >
      {isLoading ? (
        <div className="space-y-2 px-3 py-2.5">
          {[0, 1, 2].map((i) => (
            <div key={i} className="h-4 animate-pulse rounded bg-muted/40" />
          ))}
        </div>
      ) : extensions.length === 0 || used.length === 0 ? (
        <p className="px-3 py-2.5 text-xs text-foreground-subtle">
          {t("contextPanel.empty.noExtensions")}
        </p>
      ) : (
        <div>
          <div className="max-h-56 space-y-3 overflow-y-auto px-3 py-2">
            {renderSection(used)}
          </div>
        </div>
      )}
    </Widget>
  );
}
