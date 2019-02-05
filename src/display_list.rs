use webrender::api::{
    DisplayListBuilder, ImageData, ImageDescriptor, ImageKey, RenderApi, TileSize, Transaction,
};

pub struct DisplayList {
    dl: DisplayListBuilder,
    xact: Transaction,
    api: RenderApi,
}

impl DisplayList {
    pub fn new_image(
        &mut self,
        desc: ImageDescriptor,
        data: ImageData,
        tiling: Option<TileSize>,
    ) -> ImageKey {
        let key = self.api.generate_image_key();
        self.xact.add_image(key, desc, data, tiling);
        key
    }
}

impl std::ops::Deref for DisplayList {
    type Target = DisplayListBuilder;

    fn deref(&self) -> &Self::Target {
        &self.dl
    }
}

impl std::ops::DerefMut for DisplayList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dl
    }
}
