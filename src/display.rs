use embedded_graphics::mono_font::ascii::FONT_7X13_BOLD;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Point, Primitive, Size},
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use esp_hal::i2c::master::I2c;
use ssd1306::{
    mode::BufferedGraphicsModeAsync, prelude::I2CInterface, size::DisplaySize128x64, Ssd1306Async,
};

type DisplayType<'a> = Ssd1306Async<
    I2CInterface<I2c<'a, esp_hal::Async>>,
    DisplaySize128x64,
    BufferedGraphicsModeAsync<DisplaySize128x64>,
>;

pub struct Display<'a> {
    display_module: DisplayType<'a>,
}

impl<'a> Display<'a> {
    pub fn new(display_module: DisplayType<'a>) -> Self {
        Self { display_module }
    }

    pub async fn acccess_denied(&mut self) {
        self.display_module.clear(BinaryColor::On).unwrap();

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::Off)
            .stroke_width(3)
            .build();
        Rectangle::new(Point::new(8, 25), Size::new(112, 20))
            .into_styled(style)
            .draw(&mut self.display_module)
            .unwrap();

        let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::Off);
        Text::new("Access Denied", Point::new(18, 38), style)
            .draw(&mut self.display_module)
            .unwrap();
        self.display_module.flush().await.unwrap();
    }

    pub async fn acccess_granted(&mut self) {
        self.display_module.clear(BinaryColor::Off).unwrap();

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(3)
            .build();

        Rectangle::new(Point::new(8, 25), Size::new(112, 20))
            .into_styled(style)
            .draw(&mut self.display_module)
            .unwrap();

        let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
        Text::new("Access Granted", Point::new(16, 38), style)
            .draw(&mut self.display_module)
            .unwrap();
        self.display_module.flush().await.unwrap();
    }

    pub async fn wait_for_auth(&mut self) {
        self.display_module.clear(BinaryColor::Off).unwrap();
        let style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
        Text::new("Scan Your Tag...", Point::new(6, 38), style)
            .draw(&mut self.display_module)
            .unwrap();
        self.display_module.flush().await.unwrap();
    }
}
