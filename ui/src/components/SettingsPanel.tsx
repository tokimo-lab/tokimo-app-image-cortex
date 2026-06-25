import { useEffect, useMemo, useState } from "react";
import {
  Input,
  Select,
  SettingGroup,
  SettingRow,
  Spin,
  StickySaveBar,
} from "@tokimo/ui";
import { type AiSettings, api, type GeoSettings } from "../api/client";

interface Props {
  t: (key: string) => string;
}

const GEO_PROVIDER_OPTIONS = [
  { value: "amap", label: "Amap (高德)" },
  { value: "qqmap", label: "QQ Map (腾讯)" },
  { value: "tianditu", label: "Tianditu (天地图)" },
  { value: "mapbox", label: "Mapbox" },
  { value: "maptiler", label: "MapTiler" },
];

function getGeoApiKey(geo: GeoSettings): string {
  switch (geo.provider) {
    case "qqmap":
      return geo.qqmapApiKey ?? "";
    case "tianditu":
      return geo.tiandituServerKey ?? "";
    case "mapbox":
      return geo.mapboxAccessToken ?? "";
    case "maptiler":
      return geo.maptilerApiKey ?? "";
    case "amap":
    default:
      return geo.amapApiKey ?? "";
  }
}

function setGeoApiKey(geo: GeoSettings, value: string): GeoSettings {
  const apiKey = value || null;
  switch (geo.provider) {
    case "qqmap":
      return { ...geo, qqmapApiKey: apiKey };
    case "tianditu":
      return { ...geo, tiandituServerKey: apiKey };
    case "mapbox":
      return { ...geo, mapboxAccessToken: apiKey };
    case "maptiler":
      return { ...geo, maptilerApiKey: apiKey };
    case "amap":
    default:
      return { ...geo, amapApiKey: apiKey };
  }
}

export function SettingsPanel({ t }: Props) {
  const [geo, setGeo] = useState<GeoSettings | null>(null);
  const [ai, setAi] = useState<AiSettings | null>(null);
  const [initialGeo, setInitialGeo] = useState<GeoSettings | null>(null);
  const [initialAi, setInitialAi] = useState<AiSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    Promise.all([api.getGeoSettings(), api.getAiSettings()])
      .then(([g, a]) => {
        setGeo(g);
        setAi(a);
        setInitialGeo(g);
        setInitialAi(a);
      })
      .finally(() => setLoading(false));
  }, []);

  const geoDirty = useMemo(
    () =>
      geo != null &&
      initialGeo != null &&
      JSON.stringify(geo) !== JSON.stringify(initialGeo),
    [geo, initialGeo],
  );

  const aiDirty = useMemo(
    () =>
      ai != null &&
      initialAi != null &&
      JSON.stringify(ai) !== JSON.stringify(initialAi),
    [ai, initialAi],
  );

  const dirty = geoDirty || aiDirty;

  const handleSaveSettings = async () => {
    if (!dirty) return;
    setSaving(true);
    try {
      if (geo && geoDirty) {
        await api.updateGeoSettings(geo);
        setInitialGeo(geo);
      }
      if (ai && aiDirty) {
        await api.updateAiSettings(ai);
        setInitialAi(ai);
      }
    } finally {
      setSaving(false);
    }
  };

  const handleResetSettings = () => {
    setGeo(initialGeo);
    setAi(initialAi);
  };

  if (loading) {
    return (
      <div className="flex h-32 items-center justify-center text-fg-muted">
        <Spin />
      </div>
    );
  }

  return (
    <div className="relative flex min-h-full flex-col">
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

      <div className={`w-full max-w-3xl space-y-6 ${dirty ? "pb-20" : ""}`}>
        {geo && (
          <SettingGroup
            title={t("geoSettings")}
            desc={t("geoSettingsDescription")}
          >
            <SettingRow label={t("provider")} desc={t("providerDesc")}>
              <Select
                value={geo.provider}
                onChange={(provider) =>
                  setGeo({ ...geo, provider: String(provider) })
                }
                options={GEO_PROVIDER_OPTIONS}
                className="w-56"
              />
            </SettingRow>
            <SettingRow
              orientation="vertical"
              label={t("apiKey")}
              desc={t("apiKeyDesc")}
            >
              <Input.Password
                value={getGeoApiKey(geo)}
                onChange={(e) => setGeo(setGeoApiKey(geo, e.target.value))}
                className="w-full max-w-md"
              />
            </SettingRow>
          </SettingGroup>
        )}

        {ai && (
          <SettingGroup
            title={t("aiSettings")}
            desc={t("aiSettingsDescription")}
          >
            <SettingRow label="OCR" desc={t("ocrDesc")}>
              <Input
                value={ai.ocrModelName}
                onChange={(e) =>
                  setAi({ ...ai, ocrModelName: e.target.value })
                }
                className="w-56"
              />
            </SettingRow>
            <SettingRow label={t("ocrAuxModel")} desc={t("ocrAuxModelDesc")}>
              <Input
                value={ai.ocrAuxModelName ?? ""}
                onChange={(e) =>
                  setAi({
                    ...ai,
                    ocrAuxModelName: e.target.value || null,
                  })
                }
                className="w-56"
              />
            </SettingRow>
          </SettingGroup>
        )}
      </div>

      <StickySaveBar
        dirty={dirty}
        loading={saving}
        onSave={handleSaveSettings}
        onReset={handleResetSettings}
        message={t("unsavedSettings")}
        saveLabel={t("save")}
        resetLabel={t("reset")}
      />
    </div>
  );
}
