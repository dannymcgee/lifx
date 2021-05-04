#[derive(Clone, Debug)]
pub struct ProductInfo {
	pub name: &'static str,
	pub color: bool,
	pub infrared: bool,
	pub multizone: bool,
	pub chain: bool,
}

/// Look up info about what a LIFX product supports.
///
/// You can get the vendor and product IDs from a bulb by receiving a [Message::StateVersion] message
///
/// Data is taken from https://github.com/LIFX/products/blob/master/products.json
#[rustfmt::skip]
pub fn get_product_info(vendor: u32, product: u32) -> Option<&'static ProductInfo> {
	match (vendor, product) {
		(1,  1) => Some(&ProductInfo { name: "Original 1000",                color: true,  infrared: false, multizone: false, chain: false}),
		(1,  3) => Some(&ProductInfo { name: "Color 650",                    color: true,  infrared: false, multizone: false, chain: false}),
		(1, 10) => Some(&ProductInfo { name: "White 800 (Low Voltage)",      color: false, infrared: false, multizone: false, chain: false}),
		(1, 11) => Some(&ProductInfo { name: "White 800 (High Voltage)",     color: false, infrared: false, multizone: false, chain: false}),
		(1, 18) => Some(&ProductInfo { name: "White 900 BR30 (Low Voltage)", color: false, infrared: false, multizone: false, chain: false}),
		(1, 20) => Some(&ProductInfo { name: "Color 1000 BR30",              color: true,  infrared: false, multizone: false, chain: false}),
		(1, 22) => Some(&ProductInfo { name: "Color 1000",                   color: true,  infrared: false, multizone: false, chain: false}),
		(1, 27) => Some(&ProductInfo { name: "LIFX A19",                     color: true,  infrared: false, multizone: false, chain: false}),
		(1, 28) => Some(&ProductInfo { name: "LIFX BR30",                    color: true,  infrared: false, multizone: false, chain: false}),
		(1, 29) => Some(&ProductInfo { name: "LIFX+ A19",                    color: true,  infrared: true,  multizone: false, chain: false}),
		(1, 30) => Some(&ProductInfo { name: "LIFX+ BR30",                   color: true,  infrared: true,  multizone: false, chain: false}),
		(1, 31) => Some(&ProductInfo { name: "LIFX Z",                       color: true,  infrared: false, multizone: true,  chain: false}),
		(1, 32) => Some(&ProductInfo { name: "LIFX Z 2",                     color: true,  infrared: false, multizone: true,  chain: false}),
		(1, 36) => Some(&ProductInfo { name: "LIFX Downlight",               color: true,  infrared: false, multizone: false, chain: false}),
		(1, 37) => Some(&ProductInfo { name: "LIFX Downlight",               color: true,  infrared: false, multizone: false, chain: false}),
		(1, 38) => Some(&ProductInfo { name: "LIFX Beam",                    color: true,  infrared: false, multizone: true,  chain: false}),
		(1, 43) => Some(&ProductInfo { name: "LIFX A19",                     color: true,  infrared: false, multizone: false, chain: false}),
		(1, 44) => Some(&ProductInfo { name: "LIFX BR30",                    color: true,  infrared: false, multizone: false, chain: false}),
		(1, 45) => Some(&ProductInfo { name: "LIFX+ A19",                    color: true,  infrared: true,  multizone: false, chain: false}),
		(1, 46) => Some(&ProductInfo { name: "LIFX+ BR30",                   color: true,  infrared: true,  multizone: false, chain: false}),
		(1, 49) => Some(&ProductInfo { name: "LIFX Mini",                    color: true,  infrared: false, multizone: false, chain: false}),
		(1, 50) => Some(&ProductInfo { name: "LIFX Mini Day and Dusk",       color: false, infrared: false, multizone: false, chain: false}),
		(1, 51) => Some(&ProductInfo { name: "LIFX Mini White",              color: false, infrared: false, multizone: false, chain: false}),
		(1, 52) => Some(&ProductInfo { name: "LIFX GU10",                    color: true,  infrared: false, multizone: false, chain: false}),
		(1, 55) => Some(&ProductInfo { name: "LIFX Tile",                    color: true,  infrared: false, multizone: false, chain: true}),
		(1, 59) => Some(&ProductInfo { name: "LIFX Mini Color",              color: true,  infrared: false, multizone: false, chain: false}),
		(1, 60) => Some(&ProductInfo { name: "LIFX Mini Day and Dusk",       color: false, infrared: false, multizone: false, chain: false}),
		(1, 61) => Some(&ProductInfo { name: "LIFX Mini White",              color: false, infrared: false, multizone: false, chain: false}),
		(_, _)  => None
	}
}
