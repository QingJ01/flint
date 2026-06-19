import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { zh, type MessageKey } from "./zh";
import { en } from "./en";

/** Push the locale to the backend so Rust-produced strings (diagnostics,
 *  install logs) match the UI language. Best-effort — ignore failures. */
function syncBackendLocale(locale: Locale) {
  void invoke("set_locale", { locale }).catch(() => {});
}

export type Locale = "zh" | "en";

const DICTS: Record<Locale, Record<MessageKey, string>> = { zh, en };
const STORAGE_KEY = "flint.locale";

/** Initial locale: stored choice → system language → English. */
function initialLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "zh" || stored === "en") return stored;
  return navigator.language?.toLowerCase().startsWith("zh") ? "zh" : "en";
}

type TFn = (key: MessageKey, vars?: Record<string, string | number>) => string;

type I18nValue = {
  locale: Locale;
  setLocale: (l: Locale) => void;
  t: TFn;
};

const I18nContext = createContext<I18nValue | null>(null);

/** Substitute `{name}` placeholders in a template with `vars`. */
function interpolate(template: string, vars?: Record<string, string | number>): string {
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_m, k) =>
    k in vars ? String(vars[k]) : `{${k}}`,
  );
}

export function I18nProvider(props: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(initialLocale);

  // Sync the initial locale to the backend once on mount.
  useEffect(() => {
    syncBackendLocale(locale);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const setLocale = useCallback((l: Locale) => {
    setLocaleState(l);
    localStorage.setItem(STORAGE_KEY, l);
    syncBackendLocale(l);
  }, []);

  const t = useCallback<TFn>(
    (key, vars) => {
      // Missing key falls back to the zh dict, then to the raw key — so an
      // un-translated string is still readable rather than blank.
      const dict = DICTS[locale];
      const template = dict[key] ?? zh[key] ?? key;
      return interpolate(template, vars);
    },
    [locale],
  );

  const value = useMemo<I18nValue>(() => ({ locale, setLocale, t }), [locale, setLocale, t]);
  return <I18nContext.Provider value={value}>{props.children}</I18nContext.Provider>;
}

export function useI18n(): I18nValue {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within <I18nProvider>");
  return ctx;
}

/** Convenience hook when you only need the `t` function. */
export function useT(): TFn {
  return useI18n().t;
}
