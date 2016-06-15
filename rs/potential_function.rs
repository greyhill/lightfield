extern crate num;
extern crate toml;
use self::num::{Float, ToPrimitive, FromPrimitive};
use self::toml::*;
use serialize::*;
use cl_traits::*;

/// Convex one-dimensional potential function (loss)
#[derive(Clone, Debug)]
pub enum PotentialFunction<F: Float> {
    Quad,
    Abs,
    Fair(F),
}

impl<F: Float> ClHeader for PotentialFunction<F> {
    fn header() -> &'static str {
        include_str!("../cl/potential_function_f32.opencl")
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize 
for PotentialFunction<F> {
    fn from_map(map: &Table) -> Option<Self> {
        if let Some(&Value::String(ref typ)) = map.get("type") {
            match &typ[..] {
                "quad" => Some(PotentialFunction::Quad),
                "abs" => Some(PotentialFunction::Abs),
                "fair" => {
                    if let Some(&Value::Float(delta)) = map.get("delta") {
                        Some(PotentialFunction::Fair(F::from_f64(delta).unwrap()))
                    } else {
                        None
                    }
                },
                _ => None,
            }
        } else {
            None
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        match self {
            &PotentialFunction::Quad => {
                tr.insert("type".to_string(), Value::String("quad".to_string()));
            },
            &PotentialFunction::Abs => {
                tr.insert("type".to_string(), Value::String("abs".to_string()));
            },
            &PotentialFunction::Fair(ref delta) => {
                tr.insert("type".to_string(), Value::String("fair".to_string()));
                tr.insert("delta".to_string(), Value::Float(F::to_f64(delta).unwrap()));
            }
        };
        tr
    }
}

#[test]
fn test_read_potential_function() {
    let test_quad = r#"
    type = 'quad'
    "#;
    let mut parser = Parser::new(test_quad);
    let mut pf = PotentialFunction::<f32>::from_map(&parser.parse().unwrap()).unwrap();
    if let PotentialFunction::Quad = pf {
    } else {
        assert!(false);
    }

    let test_abs = r#"
    type = 'abs'
    "#;
    parser = Parser::new(test_abs);
    pf = PotentialFunction::from_map(&parser.parse().unwrap()).unwrap();
    if let PotentialFunction::Abs = pf {
    } else {
        assert!(false);
    }

    let test_fair = r#"
    type = 'fair'
    delta = 5.0
    "#;
    parser = Parser::new(test_fair);
    pf = PotentialFunction::from_map(&parser.parse().unwrap()).unwrap();
    if let PotentialFunction::Fair(f) = pf {
        assert_eq!(f, 5.0);
    } else {
        assert!(false);
    }
}

