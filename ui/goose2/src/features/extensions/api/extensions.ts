import { getClient } from "@/shared/api/acpConnection";
import type {
  ExtensionConfig,
  ExtensionEntry,
  SessionExtensionStatus,
} from "../types";

export function nameToKey(name: string): string {
  return name
    .replace(/\s/g, "")
    .replace(/[^a-zA-Z0-9_-]/g, "_")
    .toLowerCase();
}

export async function listExtensions(): Promise<ExtensionEntry[]> {
  const client = await getClient();
  const response = await client.goose.GooseConfigExtensions({});
  return response.extensions as ExtensionEntry[];
}

export async function listSessionExtensionStatus(
  sessionId: string,
): Promise<SessionExtensionStatus[]> {
  const client = await getClient();
  const response = await client.extMethod("_goose/session/extensions/status", {
    sessionId,
  });
  return (response.extensions ?? []) as SessionExtensionStatus[];
}

export async function addExtension(
  name: string,
  extensionConfig: ExtensionConfig,
): Promise<void> {
  const client = await getClient();
  await client.goose.GooseConfigExtensionsAdd({
    name,
    extensionConfig,
    enabled: true,
  });
}

export async function removeExtension(configKey: string): Promise<void> {
  const client = await getClient();
  await client.goose.GooseConfigExtensionsRemove({ configKey });
}
