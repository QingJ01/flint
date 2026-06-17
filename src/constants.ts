// Shared constants for the Flint UI. Mirror data lives here (not in
// Rust) because the dropdowns drive the Rust commands by URL string —
// Rust doesn't need to enumerate the registry list at runtime.

import type { ParameterOption } from "./types";

export const categoryLabel: Record<"runtime" | "ai-tool", string> = {
  runtime: "运行时与基础工具",
  "ai-tool": "AI 编程工具",
};

export const categoryOrder: ("runtime" | "ai-tool")[] = ["runtime", "ai-tool"];

export const NPM_MIRRORS: ParameterOption[] = [
  { value: "https://registry.npmjs.org/", label: "官方源 (npmjs.org)" },
  { value: "https://registry.npmmirror.com/", label: "淘宝镜像 (npmmirror.com)" },
  { value: "https://mirrors.huaweicloud.com/repository/npm/", label: "华为云镜像" },
  { value: "https://mirrors.cloud.tencent.com/npm/", label: "腾讯云镜像" },
];

export const PIP_MIRRORS: ParameterOption[] = [
  { value: "https://pypi.org/simple", label: "官方源 (PyPI)" },
  { value: "https://pypi.tuna.tsinghua.edu.cn/simple", label: "清华源" },
  { value: "https://mirrors.aliyun.com/pypi/simple", label: "阿里源" },
  { value: "https://mirrors.huaweicloud.com/repository/pypi/simple", label: "华为云镜像" },
  { value: "https://mirrors.cloud.tencent.com/pypi/simple", label: "腾讯云镜像" },
];

export const CN_NPM_HOSTS = ["npmmirror", "huaweicloud", "tencent"];
export const CN_PIP_HOSTS = ["tuna", "aliyun", "huaweicloud", "tencent"];

export function isCN(value: string | null | undefined, hosts: string[]): boolean {
  if (!value) return false;
  return hosts.some((h) => value.includes(h));
}
