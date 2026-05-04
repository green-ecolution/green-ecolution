use crate::shared::error::ValidationError;

/// Axis-aligned WGS-84 bounding box. Antimeridian crossing is rejected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    sw_lat: f64,
    sw_lng: f64,
    ne_lat: f64,
    ne_lng: f64,
}

impl BoundingBox {
    pub fn try_new(
        sw_lat: f64,
        sw_lng: f64,
        ne_lat: f64,
        ne_lng: f64,
    ) -> Result<Self, ValidationError> {
        if !(-90.0..=90.0).contains(&sw_lat) {
            return Err(ValidationError::OutOfRange {
                field: "bbox.sw_lat",
                min: -90.0,
                max: 90.0,
                got: sw_lat,
            });
        }
        if !(-90.0..=90.0).contains(&ne_lat) {
            return Err(ValidationError::OutOfRange {
                field: "bbox.ne_lat",
                min: -90.0,
                max: 90.0,
                got: ne_lat,
            });
        }
        if !(-180.0..=180.0).contains(&sw_lng) {
            return Err(ValidationError::OutOfRange {
                field: "bbox.sw_lng",
                min: -180.0,
                max: 180.0,
                got: sw_lng,
            });
        }
        if !(-180.0..=180.0).contains(&ne_lng) {
            return Err(ValidationError::OutOfRange {
                field: "bbox.ne_lng",
                min: -180.0,
                max: 180.0,
                got: ne_lng,
            });
        }
        if sw_lat >= ne_lat {
            return Err(ValidationError::InvalidFormat {
                field: "bbox",
                reason: format!("sw_lat ({sw_lat}) must be < ne_lat ({ne_lat})"),
            });
        }
        if sw_lng >= ne_lng {
            return Err(ValidationError::InvalidFormat {
                field: "bbox",
                reason: format!("sw_lng ({sw_lng}) must be < ne_lng ({ne_lng})"),
            });
        }
        Ok(Self { sw_lat, sw_lng, ne_lat, ne_lng })
    }

    pub fn sw_lat(&self) -> f64 { self.sw_lat }
    pub fn sw_lng(&self) -> f64 { self.sw_lng }
    pub fn ne_lat(&self) -> f64 { self.ne_lat }
    pub fn ne_lng(&self) -> f64 { self.ne_lng }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn accepts_valid_bbox() {
        let b = assert_ok!(BoundingBox::try_new(54.7, 9.4, 54.8, 9.5));
        assert_eq!(b.sw_lat(), 54.7);
        assert_eq!(b.sw_lng(), 9.4);
        assert_eq!(b.ne_lat(), 54.8);
        assert_eq!(b.ne_lng(), 9.5);
    }

    #[test]
    fn rejects_lat_out_of_range() {
        assert_err!(BoundingBox::try_new(-91.0, 0.0, 0.0, 0.0));
        assert_err!(BoundingBox::try_new(0.0, 0.0, 91.0, 0.0));
    }

    #[test]
    fn rejects_lng_out_of_range() {
        assert_err!(BoundingBox::try_new(0.0, -181.0, 0.0, 0.0));
        assert_err!(BoundingBox::try_new(0.0, 0.0, 0.0, 181.0));
    }

    #[test]
    fn rejects_sw_lat_not_below_ne_lat() {
        assert_err!(BoundingBox::try_new(54.8, 9.4, 54.7, 9.5));
        assert_err!(BoundingBox::try_new(54.7, 9.4, 54.7, 9.5));
    }

    #[test]
    fn rejects_sw_lng_not_below_ne_lng() {
        assert_err!(BoundingBox::try_new(54.7, 9.5, 54.8, 9.4));
        assert_err!(BoundingBox::try_new(54.7, 9.5, 54.8, 9.5));
    }
}
