use bevy::{
    prelude::*,
    render::{
        mesh::Mesh,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};

pub struct DebugCapsulesPlugin;

impl Plugin for DebugCapsulesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system());
    }
}

#[derive(RenderResources, Default)]
pub struct Capsule {
    pub start: Vec3,
    pub end: Vec3,
    pub radius: f32,
    pub color: Color,
}

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let mut pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            include_str!("capsule.vert"),
        )),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            include_str!("capsule.frag"),
        ))),
    }));

    render_graph.add_system_node("capsules", RenderResourcesNode::<Capsule>::new(true));

    render_graph
        .add_node_edge("capsules", base::node::MAIN_PASS)
        .unwrap();

    commands
        .spawn()
        .insert(meshes.add(Mesh::from(shape::Icosphere {
            radius: 1.0,
            subdivisions: 3,
        })))
        .insert(RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle.clone(),
        )]))
        .insert(Draw::default())
        .insert(base::MainPass::default())
        .insert(Visible {
            is_visible: true,
            is_transparent: true,
        })
        .insert(Capsule {
            start: (1.0, 1.0, 0.0).into(),
            end: (0.0, 2.0, 0.0).into(),
            radius: 0.5,
            color: Color::rgba(1.0, 1.0, 0.0, 0.25),
        });

    commands
        .spawn()
        .insert(meshes.add(Mesh::from(shape::Icosphere {
            radius: 1.0,
            subdivisions: 3,
        })))
        .insert(RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]))
        .insert(Draw::default())
        .insert(base::MainPass::default())
        .insert(Visible {
            is_visible: true,
            is_transparent: true,
        })
        .insert(Capsule {
            start: (2.0, 0.0, 0.0).into(),
            end: (4.0, 1.0, 0.0).into(),
            radius: 0.25,
            color: Color::rgba(1.0, 0.0, 0.0, 0.25),
        });
}
