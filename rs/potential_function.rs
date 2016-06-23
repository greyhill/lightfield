extern crate num;
extern crate toml;
extern crate byteorder;
use self::num::{Float, ToPrimitive, FromPrimitive};
use self::toml::*;
use serialize::*;
use cl_traits::*;
use self::byteorder::*;

/// Convex one-dimensional potential function (loss)
#[derive(Clone, Debug)]
pub enum PotentialFunction<F: Float> {
    Quad(F),
    Abs(F),
    Fair(F, F),
}

impl<F: Float> ClHeader for PotentialFunction<F> {
    fn header() -> &'static str {
        include_str!("../cl/potential_function_f32.opencl")
    }
}

impl<F: Float + ToPrimitive> ClBuffer for PotentialFunction<F> {
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) {
        match self {
            &PotentialFunction::Quad(ref weight) => {
                buf.write_i32::<LittleEndian>(0i32).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(weight).unwrap()).unwrap();
            }
            &PotentialFunction::Abs(ref weight) => {
                buf.write_i32::<LittleEndian>(1i32).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(weight).unwrap()).unwrap();
            }
            &PotentialFunction::Fair(ref weight, ref delta) => {
                buf.write_i32::<LittleEndian>(2i32).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(weight).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(delta).unwrap()).unwrap();
            }
        }
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for PotentialFunction<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let weight = if let Some(&Value::Float(f)) = map.get("weight") {
            F::from_f64(f).unwrap()
        } else {
            return None;
        };
        if let Some(&Value::String(ref typ)) = map.get("type") {
            match &typ[..] {
                "quad" => Some(PotentialFunction::Quad(weight)),
                "abs" => Some(PotentialFunction::Abs(weight)),
                "fair" => {
                    if let Some(&Value::Float(delta)) = map.get("delta") {
                        Some(PotentialFunction::Fair(weight, F::from_f64(delta).unwrap()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        match self {
            &PotentialFunction::Quad(ref weight) => {
                tr.insert("type".to_string(), Value::String("quad".to_string()));
                tr.insert("weight".to_string(),
                          Value::Float(F::to_f64(weight).unwrap()));
            }
            &PotentialFunction::Abs(ref weight) => {
                tr.insert("type".to_string(), Value::String("abs".to_string()));
                tr.insert("weight".to_string(),
                          Value::Float(F::to_f64(weight).unwrap()));
            }
            &PotentialFunction::Fair(ref weight, ref delta) => {
                tr.insert("type".to_string(), Value::String("fair".to_string()));
                tr.insert("delta".to_string(), Value::Float(F::to_f64(delta).unwrap()));
                tr.insert("weight".to_string(),
                          Value::Float(F::to_f64(weight).unwrap()));
            }
        };
        tr
    }
}

#[test]
fn test_read_potential_function() {
    let test_quad = r#"
    type = 'quad'
    weight = 1.0
    "#;
    let mut parser = Parser::new(test_quad);
    let mut pf = PotentialFunction::<f32>::from_map(&parser.parse().unwrap()).unwrap();
    if let PotentialFunction::Quad(w) = pf {
        assert_eq!(w, 1.0)
    } else {
        assert!(false);
    }

    let test_abs = r#"
    type = 'abs'
    weight = 2.0
    "#;
    parser = Parser::new(test_abs);
    pf = PotentialFunction::from_map(&parser.parse().unwrap()).unwrap();
    if let PotentialFunction::Abs(w) = pf {
        assert_eq!(w, 2.0);
    } else {
        assert!(false);
    }

    let test_fair = r#"
    type = 'fair'
    delta = 5.0
    weight = 3.0
    "#;
    parser = Parser::new(test_fair);
    pf = PotentialFunction::from_map(&parser.parse().unwrap()).unwrap();
    if let PotentialFunction::Fair(w, f) = pf {
        assert_eq!(w, 3.0);
        assert_eq!(f, 5.0);
    } else {
        assert!(false);
    }
}
