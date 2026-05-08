export type RuntimeModuleConfig = [modulePath: string, exportName?: string];

export interface RuntimeModulesConfig {
  i18n?: RuntimeModuleConfig;
  trans?: RuntimeModuleConfig;
  useLingui?: RuntimeModuleConfig;
}

export interface LinguiMacroOptions {
  runtimeModules?: RuntimeModulesConfig;
  useLinguiV5IdGeneration?: boolean;
  jsxPlaceholderAttribute?: string;
  jsxPlaceholderDefaults?: Record<string, string>;
  descriptorFields?: 'auto' | 'all' | 'id-only' | 'message'
}
