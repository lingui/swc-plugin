import * as BabelTypesNamespace from '@babel/types';
import { PluginObj } from '@babel/core';

type BabelTypes = typeof BabelTypesNamespace;
type ExtractedMessage = {
    id: string;
    message?: string;
    context?: string;
    origin?: Origin;
    comment?: string;
    placeholders?: Record<string, string>;
};
type ExtractPluginOpts = {
    onMessageExtracted(msg: ExtractedMessage): void;
};
type Origin = [filename: string, line: number, column?: number];
declare function export_default({ types: t }: {
    types: BabelTypes;
}): PluginObj;

export { export_default as default };
export type { ExtractPluginOpts, ExtractedMessage, Origin };
