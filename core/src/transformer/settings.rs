pub struct Settings {
    pub transform_block_scope: bool,
    pub transform_arrow: bool,
    pub transform_object: bool,
    pub transform_default_parameters: bool,
    pub transform_exponentation: bool,
    pub transform_class_properties: bool,
    pub transform_class: bool,
    pub transform_template_strings: bool,
}

impl Settings {
    pub fn target_es5() -> Settings {
        let mut settings = Settings::target_es2015();

        settings.transform_block_scope = true;
        settings.transform_arrow = true;
        settings.transform_default_parameters = true;
        settings.transform_object = true;
        settings.transform_class = true;
        settings.transform_template_strings = true;

        settings
    }

    pub fn target_es2015() -> Settings {
        let mut settings = Settings::no_transform();

        settings.transform_exponentation = true;
        settings.transform_class_properties = true;

        settings
    }

    pub fn no_transform() -> Settings {
        Settings {
            transform_block_scope: false,
            transform_arrow: false,
            transform_object: false,
            transform_default_parameters: false,
            transform_exponentation: false,
            transform_class_properties: false,
            transform_class: false,
            transform_template_strings: false,
        }
    }
}
