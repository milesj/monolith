// Automatically generated by schematic. DO NOT MODIFY!

/* eslint-disable */

export interface PartialTemplateVariableBoolSetting {
	default?: boolean | null;
	prompt?: string | null;
	required?: boolean | null;
	type?: 'boolean' | null;
}

export interface PartialTemplateVariableEnumValueConfig {
	label?: string | null;
	value?: string | null;
}

export type TemplateVariableEnumValue = string | TemplateVariableEnumValueConfig;

export interface PartialTemplateVariableEnumSetting {
	default?: string | null;
	multiple?: boolean | null;
	prompt?: string | null;
	type?: 'enum' | null;
	values?: TemplateVariableEnumValue[] | null;
}

export interface PartialTemplateVariableNumberSetting {
	default?: number | null;
	prompt?: string | null;
	required?: boolean | null;
	type?: 'number' | null;
}

export interface PartialTemplateVariableStringSetting {
	default?: string | null;
	prompt?: string | null;
	required?: boolean | null;
	type?: 'string' | null;
}

export type TemplateVariable = TemplateVariableBoolSetting | TemplateVariableEnumSetting | TemplateVariableNumberSetting | TemplateVariableStringSetting;

export interface PartialTemplateConfig {
	$schema?: string | null;
	description?: string | null;
	title?: string | null;
	variables?: Record<string, TemplateVariable> | null;
}

export interface PartialTemplateFrontmatterConfig {
	$schema?: string | null;
	force?: boolean | null;
	skip?: boolean | null;
	to?: string | null;
}

export interface TemplateVariableBoolSetting {
	default: boolean;
	prompt: string | null;
	required: boolean | null;
	type: 'boolean';
}

export interface TemplateVariableEnumValueConfig {
	label: string;
	value: string;
}

export interface TemplateVariableEnumSetting {
	default: string;
	multiple: boolean | null;
	prompt: string;
	type: 'enum';
	values: TemplateVariableEnumValue[];
}

export interface TemplateVariableNumberSetting {
	default: number;
	prompt: string | null;
	required: boolean | null;
	type: 'number';
}

export interface TemplateVariableStringSetting {
	default: string;
	prompt: string | null;
	required: boolean | null;
	type: 'string';
}

export interface TemplateConfig {
	$schema: string;
	description: string;
	title: string;
	variables: Record<string, TemplateVariable>;
}

export interface TemplateFrontmatterConfig {
	$schema: string;
	force: boolean;
	skip: boolean;
	to: string | null;
}
