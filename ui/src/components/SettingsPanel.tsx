import { useEffect, useState, type ReactNode } from "react";
import { Button, Input, Select, Spin, Switch } from "@tokimo/ui";
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
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    Promise.all([api.getGeoSettings(), api.getAiSettings()])
      .then(([g, a]) => {
        setGeo(g);
        setAi(a);
      })
      .finally(() => setLoading(false));
  }, []);

  const handleSaveGeo = async () => {
    if (!geo) return;
    setSaving(true);
    try {
      await api.updateGeoSettings(geo);
    } finally {
      setSaving(false);
    }
  };

  const handleSaveAi = async () => {
    if (!ai) return;
    setSaving(true);
    try {
      await api.updateAiSettings(ai);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex h-32 items-center justify-center text-fg-muted">
        <Spin />
      </div>
    );
  }

  return (
    <div className="flex max-w-2xl flex-col gap-6">
      {geo && (
        <section className="space-y-3">
          <h3 className="text-sm font-semibold text-fg-primary">
            {t("geoSettings")}
          </h3>
          <SettingRow label={t("enabled")}>
            <Switch
              checked={geo.enabled}
              onChange={(enabled) => setGeo({ ...geo, enabled })}
            />
          </SettingRow>
          <SettingRow label={t("provider")}>
            <Select
              value={geo.provider}
              onChange={(provider) =>
                setGeo({ ...geo, provider: String(provider) })
              }
              options={GEO_PROVIDER_OPTIONS}
              className="w-56"
            />
          </SettingRow>
          <SettingRow label={t("apiKey")}>
            <Input.Password
              value={getGeoApiKey(geo)}
              onChange={(e) => setGeo(setGeoApiKey(geo, e.target.value))}
              className="w-72"
            />
          </SettingRow>
          <Button
            variant="primary"
            size="small"
            onClick={handleSaveGeo}
            loading={saving}
          >
            {t("save")}
          </Button>
        </section>
      )}

      {ai && (
        <section className="space-y-3">
          <h3 className="text-sm font-semibold text-fg-primary">
            {t("aiSettings")}
          </h3>
          <SettingRow label="OCR">
            <Switch
              checked={ai.ocrEnabled}
              onChange={(ocrEnabled) => setAi({ ...ai, ocrEnabled })}
            />
          </SettingRow>
          <SettingRow label="Face">
            <Switch
              checked={ai.faceEnabled}
              onChange={(faceEnabled) => setAi({ ...ai, faceEnabled })}
            />
          </SettingRow>
          <SettingRow label="CLIP">
            <Switch
              checked={ai.clipEnabled}
              onChange={(clipEnabled) => setAi({ ...ai, clipEnabled })}
            />
          </SettingRow>
          <Button
            variant="primary"
            size="small"
            onClick={handleSaveAi}
            loading={saving}
          >
            {t("save")}
          </Button>
        </section>
      )}
    </div>
  );
}

function SettingRow({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="flex min-h-8 items-center justify-between gap-4 text-xs">
      <span className="text-fg-secondary">{label}</span>
      <div className="flex min-w-0 justify-end">{children}</div>
    </div>
  );
}
