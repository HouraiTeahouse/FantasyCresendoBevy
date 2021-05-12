/// This is a fork of the bevy_debug_lines plugin that has no long term lines.
/// All lines last only one frame.
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::{Mesh, VertexAttributeValues};
use bevy::render::pipeline::{
    PipelineDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline,
};
use bevy::render::render_graph::base;
use bevy::render::render_graph::{AssetRenderResourcesNode, RenderGraph};
use bevy::render::renderer::RenderResources;
use bevy::render::shader::{ShaderStage, ShaderStages};

pub struct DebugLinesPlugin;

impl Plugin for DebugLinesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<LineShader>()
            .init_resource::<DebugLines>()
            .add_startup_system(setup.system())
            .add_system(draw_lines.system());
    }
}

/// Maximum number of unique lines to draw at once.
pub const MAX_LINES: usize = 128000;
/// Maximum number of points.
pub const MAX_POINTS: usize = MAX_LINES * 2;

fn create_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    let positions = vec![[0.0, 0.0, 0.0]; MAX_LINES * 2];
    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );

    mesh
}

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineShader>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let mut p = PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            include_str!("line.vert"),
        )),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            include_str!("line.frag"),
        ))),
    });

    // Disable backface culling (enable two sided rendering).
    p.primitive = PrimitiveState {
        cull_mode: None,
        ..Default::default()
    };

    // Create new shader pipeline.
    let pipeline_handle = pipelines.add(p);

    render_graph.add_system_node(
        "line_shader",
        AssetRenderResourcesNode::<LineShader>::new(false),
    );

    render_graph
        .add_node_edge("line_shader", base::node::MAIN_PASS)
        .unwrap();

    let pipes = RenderPipelines::from_pipelines(vec![RenderPipeline::new(pipeline_handle)]);

    let mesh = create_mesh();
    let shader = materials.add(LineShader {
        num_lines: 0,
        points: vec![Vec4::ZERO; MAX_POINTS],
        colors: vec![Color::WHITE.into(); MAX_POINTS],
    });

    commands
        .spawn_bundle(MeshBundle {
            mesh: meshes.add(mesh),
            render_pipelines: pipes,
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        })
        .insert(shader);

    info!("Loaded debug lines plugin.");
}

/// Shader data, passed to the GPU.
#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "f093e7c5-634c-45f8-a2af-7fcd0245f259"]
pub struct LineShader {
    pub num_lines: u32, // max number of lines: see: MAX_LINES.
    // I don't love having 2 buffers here.  It would be cleaner if we can do a custom line structure.
    // We should also consider the memory imprint here.  We should maybe instead allow a predefined
    // set of colors which would dramatically reduce that.
    #[render_resources(buffer)]
    pub points: Vec<Vec4>,
    #[render_resources(buffer)]
    pub colors: Vec<Vec4>,
}

/// A single line, usually initialized by helper methods on `DebugLines` instead of directly.
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Color,
}

impl Line {
    pub fn new(start: Vec3, end: Vec3, color: Color) -> Self {
        Self { start, end, color }
    }
}

/// Bevy resource providing facilities to draw lines.
///
/// # Usage
/// ```
/// // Draws 3 horizontal lines, which disappear after 1 frame.
/// fn some_system(mut lines: ResMut<DebugLines>) {
///     lines.line(Vec3::new(-1.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 0.0), 0.0);
///     lines.line_colored(
///         Vec3::new(-1.0, 0.0, 0.0),
///         Vec3::new(1.0, 0.0, 0.0),
///         Color::WHITE
///     );
/// }
/// ```
///
/// # Properties
///
/// * `lines` - A `Vec` of `Line`s that is **cleared by the system every frame**.
/// Normally you don't want to touch this, and it may go private in future releases.
/// * `user_lines` - A Vec of `Line`s that is **not cleared by the system every frame**.
/// Use this for inserting persistent lines and generally having control over how lines are collected.
pub struct DebugLines {
    pub lines: Vec<Line>,
    pub user_lines: Vec<Line>,
}

impl Default for DebugLines {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            user_lines: Vec::new(),
        }
    }
}

impl DebugLines {
    /// Draw a line in world space, or update an existing line
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the line in world space
    /// * `end` - The end of the line in world space
    pub fn line(&mut self, start: Vec3, end: Vec3) {
        self.line_colored(start, end, Color::WHITE);
    }

    /// Draw a line in world space with a specified color, or update an existing line
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the line in world space
    /// * `end` - The end of the line in world space
    /// * `color` - Line color
    pub fn line_colored(&mut self, start: Vec3, end: Vec3, color: Color) {
        let line = Line::new(start, end, color);

        // If we are at maximum capacity, we push the first line out.
        if self.lines.len() == MAX_LINES {
            //bevy::log::warn!("Hit max lines, so replaced most recent line.");
            self.lines.pop();
        }

        self.lines.push(line);
    }
}

fn draw_lines(
    mut assets: ResMut<Assets<LineShader>>,
    mut lines: ResMut<DebugLines>,
    query: Query<&Handle<LineShader>>,
) {
    for line_handle in query.iter() {
        // This could probably be faster if we can simplify to a memcpy instead.
        if let Some(shader) = assets.get_mut(line_handle) {
            let mut i = 0;
            let all_lines = lines.lines.iter().chain(lines.user_lines.iter());
            for line in all_lines {
                shader.points[i] = line.start.extend(0.0);
                shader.points[i + 1] = line.end.extend(0.0);
                shader.colors[i] = line.color.as_rgba_f32().into();
                shader.colors[i + 1] = line.color.as_rgba_f32().into();

                i += 2;
            }

            let count = lines.lines.len() + lines.user_lines.len();
            let size = if count > MAX_LINES {
                bevy::log::warn!(
                    "DebugLines: Maximum number of lines exceeded: line count: {}, max lines: {}",
                    count,
                    MAX_LINES
                );
                MAX_LINES
            } else {
                count
            };

            shader.num_lines = size as u32; // Minimum size to send to shader is 4 bytes.
        }
    }

    lines.lines.clear();
}
