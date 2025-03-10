use glib::object::Cast;
use gtk::prelude::WidgetExt;

use super::PwPeakMeter;

pub enum PeakMeterType {
    None,
    Basic,
    Led
}


#[derive(Debug)]
enum PeakMeterWidget {
    None,
    Basic(gtk::LevelBar),
    Led(PwPeakMeter)
}

#[derive(Debug)]
pub struct PeakMeterAbstraction {
    peak_meter: PeakMeterWidget,
}

impl PeakMeterAbstraction {
    pub fn new(peak_meter_type: PeakMeterType) -> PeakMeterAbstraction {
        match peak_meter_type {
            PeakMeterType::Basic => {
                let level_bar = gtk::LevelBar::new();
                level_bar.set_mode(gtk::LevelBarMode::Continuous);
                level_bar.set_min_value(0.0);
                level_bar.set_max_value(1.0);
    
                level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_LOW, 0.0);
                level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_HIGH, 0.0);
                level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_FULL, 1.0);

                Self { peak_meter: PeakMeterWidget::Basic(level_bar)}
            },
            PeakMeterType::Led => {
                let widget = PwPeakMeter::new();
                Self { peak_meter: PeakMeterWidget::Led(widget)}
            },
            PeakMeterType::None => Self { peak_meter: PeakMeterWidget::None }
        }
    }

    pub fn set_visible(&self, visible: bool) {
        match &self.peak_meter {
            PeakMeterWidget::Basic(level_bar) => level_bar.set_visible(visible),
            PeakMeterWidget::Led(pw_peak_meter) => pw_peak_meter.set_visible(visible),
            PeakMeterWidget::None => {},
        }
    }

    pub fn set_level(&self, level: f32) {
        match &self.peak_meter {
            PeakMeterWidget::Basic(level_bar) => level_bar.set_value(level as f64),
            PeakMeterWidget::Led(pw_peak_meter) => pw_peak_meter.set_level(level),
            PeakMeterWidget::None => {},
        }
    }

    pub fn get_widget(&self) -> Option<&gtk::Widget> {
        match &self.peak_meter {
            PeakMeterWidget::Basic(level_bar) => Some(level_bar.upcast_ref()),
            PeakMeterWidget::Led(pw_peak_meter) => Some(pw_peak_meter.upcast_ref()),
            PeakMeterWidget::None => None,
        }
    }
}

impl Default for PeakMeterAbstraction {
    fn default() -> Self {
        Self::new(PeakMeterType::None)
    }
}
