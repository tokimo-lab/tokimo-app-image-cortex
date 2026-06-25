import { Button, SettingGroup, SettingRow } from "@tokimo/ui";
import { ExternalLink } from "lucide-react";

interface Props {
  t: (key: string) => string;
  onOpenAiModels: () => void;
}

export function SettingsPanel({ t, onOpenAiModels }: Props) {
  return (
    <div className="flex min-h-full flex-col">
      <div className="mb-6 flex items-start justify-between gap-4">
        <div>
          <h2 className="text-base font-semibold leading-tight text-fg-primary">
            {t("settingsTitle")}
          </h2>
          <p className="mt-1 max-w-2xl text-xs leading-relaxed text-fg-muted">
            {t("settingsDescription")}
          </p>
        </div>
      </div>

      <SettingGroup
        title={t("modelSettings")}
        desc={t("modelSettingsDescription")}
      >
        <SettingRow
          label={t("aiModelManagement")}
          desc={t("aiModelManagementDesc")}
        >
          <Button
            icon={<ExternalLink className="h-4 w-4" />}
            onClick={onOpenAiModels}
          >
            {t("openAiModelManagement")}
          </Button>
        </SettingRow>
      </SettingGroup>
    </div>
  );
}
