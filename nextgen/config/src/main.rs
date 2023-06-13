use moon_config::*;
use schematic::renderers::json_schema::JsonSchemaRenderer;
use schematic::renderers::typescript::TypeScriptRenderer;
// use schemars::schema_for;
// use schematic::typescript::{Output, Type, TypeScriptGenerator};
use schematic::schema::SchemaGenerator;
use std::fs;
use std::path::PathBuf;

// fn create_type_alias(name: &str) -> Output {
//     Output::Enum {
//         name,
//         fields: vec![Output::Field {
//             name: "".into(),
//             value: Type::String,
//             optional: false,
//         }],
//     }
// }

// fn generate_common() {
//     let mut generator =
//         TypeScriptGenerator::new(PathBuf::from("packages/types/src/common-config.ts"));

//     generator.add_custom(create_type_alias("Id"));
//     generator.add_custom(create_type_alias("Target"));
//     generator.add_custom(create_type_alias("FilePath"));
//     generator.add_custom(create_type_alias("GlobPath"));
//     generator.add_custom(create_type_alias("InputPath"));
//     generator.add_custom(create_type_alias("OutputPath"));
//     generator.add_enum::<LanguageType>();
//     generator.add_enum::<PlatformType>();

//     generator.generate().unwrap();
// }

fn generate_project() {
    let mut generator = SchemaGenerator::default();

    generator.add::<PartialProjectConfig>();

    generator
        .generate(
            PathBuf::from("website/static/schemas/project.json"),
            JsonSchemaRenderer::default(),
        )
        .unwrap();

    generator.add::<ProjectConfig>();

    generator
        .generate(
            PathBuf::from("packages/types/src/project-config.ts"),
            TypeScriptRenderer::default(),
        )
        .unwrap();
}

fn generate_tasks() {
    let mut generator = SchemaGenerator::default();

    generator.add::<PartialInheritedTasksConfig>();

    generator
        .generate(
            PathBuf::from("website/static/schemas/tasks.json"),
            JsonSchemaRenderer::default(),
        )
        .unwrap();

    generator.add::<InheritedTasksConfig>();

    generator
        .generate(
            PathBuf::from("packages/types/src/tasks-config.ts"),
            TypeScriptRenderer::default(),
        )
        .unwrap();
}

fn generate_template() {
    let mut generator = SchemaGenerator::default();
    generator.add::<PartialTemplateConfig>();
    generator
        .generate(
            PathBuf::from("website/static/schemas/template.json"),
            JsonSchemaRenderer::default(),
        )
        .unwrap();

    let mut generator = SchemaGenerator::default();
    generator.add::<PartialTemplateFrontmatterConfig>();
    generator
        .generate(
            PathBuf::from("website/static/schemas/template-frontmatter.json"),
            JsonSchemaRenderer::default(),
        )
        .unwrap();
}

fn generate_toolchain() {
    let mut generator = SchemaGenerator::default();

    generator.add::<PartialToolchainConfig>();

    generator
        .generate(
            PathBuf::from("website/static/schemas/toolchain.json"),
            JsonSchemaRenderer::default(),
        )
        .unwrap();

    generator.add::<ToolchainConfig>();

    generator
        .generate(
            PathBuf::from("packages/types/src/toolchain-config.ts"),
            TypeScriptRenderer::default(),
        )
        .unwrap();
}

fn generate_workspace() {
    let mut generator = SchemaGenerator::default();

    generator.add::<PartialWorkspaceConfig>();

    generator
        .generate(
            PathBuf::from("website/static/schemas/workspace.json"),
            JsonSchemaRenderer::default(),
        )
        .unwrap();

    generator.add::<WorkspaceConfig>();

    generator
        .generate(
            PathBuf::from("packages/types/src/workspace-config.ts"),
            TypeScriptRenderer::default(),
        )
        .unwrap();
}

fn main() {
    // generate_common();
    generate_project();
    generate_tasks();
    generate_template();
    // generate_toolchain();
    generate_workspace();
}
