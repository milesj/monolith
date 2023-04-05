use crate::helpers::AnyError;
use console::Term;
use itertools::Itertools;
use moon::{build_project_graph, load_workspace};
use moon_logger::{color, map_list};
use moon_terminal::{ExtendedTerm, Label};
use moon_utils::is_test_env;

pub async fn project(id: String, json: bool) -> Result<(), AnyError> {
    let mut workspace = load_workspace().await?;
    let mut project_builder = build_project_graph(&mut workspace).await?;
    project_builder.load(&id)?;

    let project_graph = project_builder.build()?;
    let project = project_graph.get(&id)?;
    let config = &project.config;

    if json {
        println!("{}", serde_json::to_string_pretty(&project)?);

        return Ok(());
    }

    let term = Term::buffered_stdout();

    term.write_line("")?;
    term.render_label(Label::Brand, &project.id)?;
    term.render_entry("Project", color::id(&project.id))?;

    if !project.aliases.is_empty() {
        term.render_entry(
            if project.aliases.len() == 1 {
                "Alias"
            } else {
                "Aliases"
            },
            map_list(&project.aliases, |alias| color::id(alias)),
        )?;
    }

    term.render_entry("Source", color::file(&project.source))?;

    // Dont show in test snapshots
    if !is_test_env() {
        term.render_entry("Root", color::path(&project.root))?;
    }

    term.render_entry("Language", term.format(&project.language))?;
    term.render_entry("Type", term.format(&project.type_of))?;

    if !config.tags.is_empty() {
        term.render_entry("Tags", map_list(&config.tags, |tag| color::id(tag)))?;
    }

    if let Some(meta) = &config.project {
        if let Some(name) = &meta.name {
            term.render_entry("Name", name)?;
        }

        term.render_entry("Description", &meta.description)?;

        if let Some(owner) = &meta.owner {
            term.render_entry("Owner", owner)?;
        }

        if let Some(maintainers) = &meta.maintainers {
            term.render_entry_list("Maintainers", maintainers)?;
        }

        if let Some(channel) = &meta.channel {
            term.render_entry("Channel", channel)?;
        }
    }

    let mut deps = vec![];

    for (dep_id, dep_cfg) in &project.dependencies {
        deps.push(format!(
            "{} {}",
            color::id(dep_id),
            color::muted_light(format!("({}, {})", dep_cfg.source, dep_cfg.scope)),
        ));
    }

    if !deps.is_empty() {
        deps.sort();

        term.write_line("")?;
        term.render_label(Label::Default, "Depends on")?;
        term.render_list(deps)?;
    }

    if !project.tasks.is_empty() {
        term.write_line("")?;
        term.render_label(Label::Default, "Tasks")?;

        for name in project.tasks.keys().sorted() {
            let task = project.tasks.get(name).unwrap();

            term.render_entry(
                name,
                color::shell(format!("{} {}", task.command, task.args.join(" "))),
            )?;
        }
    }

    if !project.file_groups.is_empty() {
        term.write_line("")?;
        term.render_label(Label::Default, "File groups")?;

        for group_name in project.file_groups.keys().sorted() {
            let mut files = vec![];
            let group = project.file_groups.get(group_name).unwrap();

            for file in &group.files {
                files.push(color::file(file));
            }

            for file in &group.globs {
                files.push(color::file(file));
            }

            term.render_entry_list(group_name, files)?;
        }
    }

    term.write_line("")?;
    term.flush()?;

    Ok(())
}
