import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { IconChevronDown, IconPlus } from "@tabler/icons-react";
import { Button } from "@/shared/ui/button";
import { SearchBar } from "@/shared/ui/SearchBar";
import { FilterRow, PageHeader } from "@/shared/ui/page-shell";
import {
  listExtensions,
  addExtension,
  removeExtension,
  nameToKey,
} from "../api/extensions";
import {
  getDisplayName,
  type ExtensionConfig,
  type ExtensionEntry,
} from "../types";
import { ExtensionItem } from "./ExtensionItem";
import { ExtensionModal } from "./ExtensionModal";

type ExtensionCategory = "appsServices" | "gooseCapabilities";

type ExtensionFilter = "all" | ExtensionCategory;

const GOOSE_CAPABILITY_TYPES = new Set(["builtin", "platform"]);

function classifyExtension(extension: ExtensionEntry): ExtensionCategory {
  if (GOOSE_CAPABILITY_TYPES.has(extension.type)) {
    return "gooseCapabilities";
  }
  return "appsServices";
}

function compareExtensionsByName(a: ExtensionEntry, b: ExtensionEntry) {
  return getDisplayName(a).localeCompare(getDisplayName(b));
}

function FilterButton({
  active,
  children,
  onClick,
}: {
  active: boolean;
  children: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <Button
      type="button"
      size="xs"
      variant={active ? "default" : "outline-flat"}
      onClick={onClick}
    >
      {children}
    </Button>
  );
}

export function ExtensionsSettings() {
  const { t } = useTranslation("settings");
  const [extensions, setExtensions] = useState<ExtensionEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [modalMode, setModalMode] = useState<"add" | "edit" | null>(null);
  const [editingExtension, setEditingExtension] =
    useState<ExtensionEntry | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [activeFilter, setActiveFilter] = useState<ExtensionFilter>("all");
  const [showGooseCapabilities, setShowGooseCapabilities] = useState(false);

  const fetchExtensions = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await listExtensions();
      setExtensions(result);
    } catch {
      setExtensions([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void fetchExtensions();
  }, [fetchExtensions]);

  const matchesSearch = useCallback(
    (ext: ExtensionEntry) => {
      if (!searchTerm) return true;
      const q = searchTerm.toLowerCase();
      const category = classifyExtension(ext);
      return (
        getDisplayName(ext).toLowerCase().includes(q) ||
        ext.name.toLowerCase().includes(q) ||
        (ext.description ?? "").toLowerCase().includes(q) ||
        t(`extensions.categories.${category}`).toLowerCase().includes(q)
      );
    },
    [searchTerm, t],
  );

  const filteredExtensions = useMemo(
    () =>
      extensions
        .filter((extension) => {
          const category = classifyExtension(extension);
          return (
            matchesSearch(extension) &&
            (activeFilter === "all" || category === activeFilter)
          );
        })
        .sort(compareExtensionsByName),
    [activeFilter, extensions, matchesSearch],
  );

  const primaryExtensions = useMemo(
    () =>
      filteredExtensions.filter(
        (ext) => classifyExtension(ext) !== "gooseCapabilities",
      ),
    [filteredExtensions],
  );

  const gooseCapabilities = useMemo(
    () =>
      filteredExtensions.filter(
        (ext) => classifyExtension(ext) === "gooseCapabilities",
      ),
    [filteredExtensions],
  );

  const visibleExtensions =
    activeFilter === "gooseCapabilities"
      ? gooseCapabilities
      : [...primaryExtensions, ...gooseCapabilities];
  const shouldShowGooseCapabilities =
    activeFilter === "gooseCapabilities" || showGooseCapabilities;
  const showGooseCapabilitiesToggle =
    activeFilter !== "gooseCapabilities" && gooseCapabilities.length > 0;

  const categoryCounts = useMemo(() => {
    const counts: Record<ExtensionCategory, number> = {
      appsServices: 0,
      gooseCapabilities: 0,
    };
    for (const extension of extensions) {
      counts[classifyExtension(extension)] += 1;
    }
    return counts;
  }, [extensions]);

  const handleConfigure = (ext: ExtensionEntry) => {
    setEditingExtension(ext);
    setModalMode("edit");
  };

  const handleSubmit = async (name: string, config: ExtensionConfig) => {
    try {
      const newKey = nameToKey(name);
      const isEdit = !!editingExtension;
      const isAdd = !editingExtension;
      const keyChanged = isEdit && editingExtension.config_key !== newKey;

      if (
        (isAdd || keyChanged) &&
        extensions.some((e) => e.config_key === newKey)
      ) {
        toast.error(t("extensions.errors.nameConflict", { name }));
        return;
      }

      await addExtension(name, config);
      if (keyChanged) {
        await removeExtension(editingExtension.config_key);
      }
      setModalMode(null);
      setEditingExtension(null);
      await fetchExtensions();
    } catch {
      toast.error(t("extensions.errors.saveFailed"));
    }
  };

  const handleDelete = async (configKey: string) => {
    try {
      await removeExtension(configKey);
      setModalMode(null);
      setEditingExtension(null);
      await fetchExtensions();
    } catch {
      toast.error(t("extensions.errors.deleteFailed"));
    }
  };

  const handleModalClose = () => {
    setModalMode(null);
    setEditingExtension(null);
  };

  const renderSection = (
    title: string,
    sectionExtensions: ExtensionEntry[],
    showTitle = true,
  ) => {
    if (sectionExtensions.length === 0) return null;
    return (
      <section className="space-y-3">
        {showTitle ? (
          <h4 className="text-sm font-normal text-foreground">{title}</h4>
        ) : null}
        <div className="grid gap-x-12 sm:grid-cols-2">
          {sectionExtensions.map((ext) => (
            <ExtensionItem
              key={ext.config_key}
              extension={ext}
              onConfigure={handleConfigure}
            />
          ))}
        </div>
      </section>
    );
  };

  return (
    <div className="space-y-8">
      <PageHeader
        title={t("extensions.title")}
        description={t("extensions.description")}
        titleClassName="font-normal text-foreground"
        actions={
          <Button
            type="button"
            variant="outline-flat"
            size="xs"
            onClick={() => {
              setEditingExtension(null);
              setModalMode("add");
            }}
          >
            <IconPlus className="size-3.5" />
            {t("extensions.addExtension")}
          </Button>
        }
      />

      <div className="space-y-3">
        <SearchBar
          value={searchTerm}
          onChange={setSearchTerm}
          placeholder={t("extensions.search")}
        />
        <FilterRow>
          <FilterButton
            active={activeFilter === "all"}
            onClick={() => setActiveFilter("all")}
          >
            {t("extensions.filters.all")}
          </FilterButton>
          {(["appsServices", "gooseCapabilities"] as ExtensionCategory[]).map(
            (category) =>
              categoryCounts[category] > 0 ? (
                <FilterButton
                  key={category}
                  active={activeFilter === category}
                  onClick={() => setActiveFilter(category)}
                >
                  {t(`extensions.categories.${category}`)}
                </FilterButton>
              ) : null,
          )}
        </FilterRow>
      </div>

      {isLoading ? (
        <div className="grid gap-x-12 sm:grid-cols-2">
          {[1, 2, 3, 4, 5, 6].map((i) => (
            <div
              key={i}
              className="h-20 animate-pulse border-b border-border-soft-divider py-4"
            >
              <div className="h-4 w-2/5 rounded bg-muted/50" />
              <div className="mt-2 h-3 w-3/5 rounded bg-muted/40" />
            </div>
          ))}
        </div>
      ) : extensions.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("extensions.empty")}</p>
      ) : visibleExtensions.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {t("extensions.noResults")}
        </p>
      ) : (
        <div className="space-y-8">
          {activeFilter !== "gooseCapabilities"
            ? renderSection(
                t("extensions.sections.extensions"),
                primaryExtensions,
                false,
              )
            : null}

          {shouldShowGooseCapabilities
            ? renderSection(
                t("extensions.sections.gooseCapabilities"),
                gooseCapabilities,
              )
            : null}

          {showGooseCapabilitiesToggle ? (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => setShowGooseCapabilities((current) => !current)}
              className="w-full text-muted-foreground"
            >
              {showGooseCapabilities
                ? t("extensions.hideGooseCapabilities")
                : t("extensions.showGooseCapabilities", {
                    count: gooseCapabilities.length,
                  })}
              {!showGooseCapabilities ? (
                <IconChevronDown className="size-3" />
              ) : null}
            </Button>
          ) : null}
        </div>
      )}

      {modalMode === "add" && (
        <ExtensionModal onSubmit={handleSubmit} onClose={handleModalClose} />
      )}

      {modalMode === "edit" && editingExtension && (
        <ExtensionModal
          extension={editingExtension}
          onSubmit={handleSubmit}
          onDelete={handleDelete}
          onClose={handleModalClose}
        />
      )}
    </div>
  );
}
