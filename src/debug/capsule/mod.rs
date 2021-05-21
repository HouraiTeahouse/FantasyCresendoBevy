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

pub struct CapsuleGenerator {
    mesh: Handle<Mesh>,
    pipelines: RenderPipelines,
}

impl CapsuleGenerator {
    pub fn create(&self) -> CapsuleBundle {
        CapsuleBundle {
            mesh: self.mesh.clone_weak(),
            pipelines: self.pipelines.clone(),
            visible: Visible {
                is_visible: true,
                is_transparent: true,
            },
            ..Default::default()
        }
    }
}

#[derive(Bundle, Default)]
pub struct CapsuleBundle {
    mesh: Handle<Mesh>,
    draw: Draw,
    visible: Visible,
    main_pass: base::MainPass,
    pipelines: RenderPipelines,
    capsule: Capsule,
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
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
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

    commands.insert_resource(CapsuleGenerator {
        mesh: meshes.add(Mesh::from(shape::Icosphere {
            radius: 1.0,
            subdivisions: 3,
        })),
        pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle.clone(),
        )]),
    });
}
