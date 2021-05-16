use crate::AppState;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadState, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use fc_core::{
    character::{state::StateMachine, CharacterAsset},
    stage::StageAsset,
};

struct FcAssetLoader<T> {
    extensions: &'static [&'static str],
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FcAssetLoader<T> {
    pub fn new(ext: &'static [&'static str]) -> Self {
        Self {
            extensions: ext,
            _phantom: std::marker::PhantomData::<T>::default(),
        }
    }
}

impl<T: serde::de::DeserializeOwned + TypeUuid + Send + Sync + 'static> AssetLoader
    for FcAssetLoader<T>
{
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let decoded = std::str::from_utf8(bytes)?;
            let custom_asset = serde_json::from_str::<T>(decoded)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}

pub struct FcMetadata {
    pub characters: Vec<Handle<CharacterAsset>>,
    pub stages: Vec<Handle<CharacterAsset>>,
}

fn load_folder<T: TypeUuid + Send + Sync + 'static>(
    folder: &str,
    asset_server: &Res<AssetServer>,
) -> Vec<Handle<T>> {
    asset_server
        .load_folder(folder)
        .unwrap_or_else(|_| panic!("Failed to load {}", folder))
        .into_iter()
        .map(|handle| handle.typed::<T>())
        .collect()
}

fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FcMetadata {
        characters: load_folder("characters", &asset_server),
        stages: load_folder("stages", &asset_server),
    })
}

fn check_loading(
    metadata: Res<FcMetadata>,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<State<AppState>>,
) {
    let ids = metadata
        .characters
        .iter()
        .map(|handle| handle.id)
        .chain(metadata.stages.iter().map(|handle| handle.id));
    for id in ids {
        match asset_server.get_load_state(id) {
            LoadState::NotLoaded => panic!("Assets failed to start loading"),
            LoadState::Loaded | LoadState::Failed => continue,
            LoadState::Loading => return,
        }
    }
    app_state
        .replace(AppState::MATCH)
        .expect("Unable to change game state.");
}

fn cleanup_loading(characters: Res<Assets<CharacterAsset>>, stages: Res<Assets<StageAsset>>) {
    for (id, character) in characters.iter() {
        info!("Loaded character: {} ({:?})", character.short_name, id);
    }
    for (id, stage) in stages.iter() {
        info!("Loaded stage: {} ({:?})", stage.name, id);
    }
    info!("Loaded characters: {}", characters.iter().count());
    info!("Loaded stages: {}", stages.iter().count());
}

pub struct FcAssetsPlugin;

impl Plugin for FcAssetsPlugin {
    fn build(&self, builder: &mut AppBuilder) {
        builder
            .add_asset::<CharacterAsset>()
            .add_asset::<StageAsset>()
            .add_asset::<StateMachine>()
            .add_asset_loader(FcAssetLoader::<CharacterAsset>::new(&["chr"]))
            .add_asset_loader(FcAssetLoader::<StageAsset>::new(&["stage"]))
            .add_system_set(
                SystemSet::on_enter(AppState::STARTUP).with_system(start_loading.system()),
            )
            .add_system_set(
                SystemSet::on_update(AppState::STARTUP).with_system(check_loading.system()),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::STARTUP).with_system(cleanup_loading.system()),
            );
    }
}
