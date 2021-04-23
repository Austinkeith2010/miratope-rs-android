use crate::translation::{name::NameType, Gender, Name};

use super::super::{GreekPrefix, Language, Options, Prefix};

/// The Spanish language.
pub struct Es;

impl GreekPrefix for Es {}

/// In Spanish, polygon names have the last vowel in their prefix accented.
/// This function places such accent.
fn last_vowel_tilde(prefix: String) -> String {
    let mut chars = prefix.chars().collect::<Vec<_>>();
    for c in chars.iter_mut().rev() {
        match c {
            'a' => {
                *c = 'á';
                break;
            }
            'e' => {
                *c = 'é';
                break;
            }
            'i' => {
                *c = 'í';
                break;
            }
            'o' => {
                *c = 'ó';
                break;
            }
            'u' => {
                *c = 'ú';
                break;
            }
            _ => {}
        }
    }

    chars.into_iter().collect()
}

impl Language for Es {
    /// The name of a nullitope.
    fn nullitope(options: Options) -> String {
        format!(
            "nul{}",
            Self::six(
                options,
                "itopo",
                "itopos",
                "itópico",
                "itópicos",
                "itópica",
                "itópicas"
            )
        )
    }

    /// The name of a point.
    fn point(options: Options) -> String {
        format!("punt{}", Self::four(options, "o", "os", "ual", "uales"))
    }

    /// The name of a dyad.
    fn dyad(options: Options) -> String {
        format!(
            "d{}",
            Self::six(options, "íada", "íadas", "iádico", "iádicos", "iádica", "iádicas")
        )
    }

    /// The name of a triangle.
    fn triangle<T: NameType>(_regular: T, options: Options) -> String {
        format!(
            "tri{}",
            Self::four(options, "ángulo", "ángulos", "angular", "angulares")
        )
    }

    /// The name of a square.
    fn square(options: Options) -> String {
        format!(
            "cuadrad{}",
            Self::six(options, "o", "os", "o", "os", "a", "as")
        )
    }

    /// The name of a rectangle.
    fn rectangle(options: Options) -> String {
        format!(
            "rect{}",
            Self::four(options, "ángulo", "ángulos", "angular", "angulares")
        )
    }

    /// The name of an orthodiagonal quadrilateral. You should probably just
    /// default this one to "tetragon," as it only exists for tracking purposes.
    fn orthodiagonal(options: Options) -> String {
        Self::generic(4, 2, options)
    }

    /// The generic name for a polytope with `n` facets in `d` dimensions.
    fn generic(n: usize, d: usize, options: Options) -> String {
        let mut prefix = Self::prefix(n);

        if d == 2 && !options.adjective {
            prefix = last_vowel_tilde(prefix);
        }

        format!("{}{}", prefix, Self::suffix(d, options))
    }

    fn pyramidal(options: Options) -> String {
        format!(
            "pir{}",
            Self::four(options, "ámide", "ámides", "amidal", "amidales")
        )
    }

    /// The name for a pyramid with a given base.
    fn pyramid<T: NameType>(base: &Name<T>, options: Options) -> String {
        format!(
            "{} {}",
            Self::pyramidal(options),
            Self::base_adj(
                base,
                Options {
                    gender: options.gender | Gender::Female,
                    ..options
                }
            )
        )
    }

    fn prismatic(options: Options) -> String {
        format!(
            "prism{}",
            Self::six(options, "a", "as", "ático", "áticos", "ática", "áticas")
        )
    }

    /// The name for a prism with a given base.
    fn prism<T: NameType>(base: &Name<T>, options: Options) -> String {
        format!(
            "{} {}",
            Self::prismatic(options),
            Self::base_adj(
                base,
                Options {
                    gender: options.gender | Gender::Male,
                    ..options
                }
            )
        )
    }

    fn tegmatic(options: Options) -> String {
        format!(
            "teg{}",
            Self::six(options, "o", "os", "mático", "máticos", "mática", "máticas")
        )
    }

    /// The name for a tegum with a given base.
    fn tegum<T: NameType>(base: &Name<T>, options: Options) -> String {
        format!(
            "{} {}",
            Self::tegmatic(options),
            Self::base_adj(
                base,
                Options {
                    gender: options.gender | Gender::Male,
                    ..options
                }
            )
        )
    }

    fn multiproduct<T: NameType>(name: &Name<T>, options: Options) -> String {
        // Gets the bases and the kind of multiproduct.
        let (bases, kind, gender) = match name {
            Name::Multipyramid(bases) => (bases, Self::pyramidal(options), Gender::Female),
            Name::Multiprism(bases) => (bases, Self::prismatic(options), Gender::Male),
            Name::Multitegum(bases) => (bases, Self::tegmatic(options), Gender::Male),
            Name::Multicomb(bases) => (
                bases,
                String::from(Self::four(
                    options,
                    "panal",
                    "panales",
                    "de panal",
                    "de panales",
                )),
                Gender::Male,
            ),
            _ => panic!("Not a product!"),
        };
        let gender = options.gender | gender;

        let n = bases.len();
        let prefix = match n {
            2 => String::from("duo"),
            3 => String::from("trio"),
            _ => Self::prefix(n),
        };
        let kind = format!("{}{}", prefix, kind);

        let mut str_bases = String::new();
        let new_options = Options { gender, ..options };
        let (last, bases) = bases.split_last().unwrap();
        for base in bases {
            str_bases.push_str(&Self::base_adj(base, new_options));
            str_bases.push('-');
        }
        str_bases.push_str(&Self::base_adj(last, new_options));

        format!("{} {}", kind, str_bases)
    }

    /// The name for a hypercube with a given rank.
    fn hypercube<T: NameType>(regular: T, rank: usize, options: Options) -> String {
        if regular.is_regular() {
            match rank {
                3 => format!(
                    "c{}",
                    Self::six(options, "ubo", "ubos", "úbico", "úbicos", "úbica", "úbicas")
                ),
                4 => format!(
                    "tesser{}",
                    Self::six(
                        options, "acto", "actos", "áctico", "ácticoa", "áctica", "ácticas"
                    )
                ),
                _ => {
                    let prefix = Self::prefix(rank).chars().collect::<Vec<_>>();

                    // Penta -> Pente, or Deca -> Deque
                    // Penta -> Pente, or Deca -> Deke
                    let (_, str0) = prefix.split_last().unwrap();
                    let (c1, str1) = str0.split_last().unwrap();

                    let suffix = Self::six(
                        options, "acto", "actos", "áctico", "ácticos", "áctica", "ácticas",
                    );
                    if *c1 == 'c' {
                        format!("{}quer{}", str1.into_iter().collect::<String>(), suffix)
                    } else {
                        format!("{}eract{}", str0.into_iter().collect::<String>(), suffix)
                    }
                }
            }
        } else {
            match rank {
                3 => format!("cuboid{}", Self::three(options, "", "s", "al")),
                _ => {
                    format!("{}block{}", Self::prefix(rank), Self::two(options, "", "s"))
                }
            }
        }
    }

    /// The name for an orthoplex with a given rank.
    fn orthoplex<T: NameType>(_regular: T, rank: usize, options: Options) -> String {
        Self::generic(2u32.pow(rank as u32) as usize, rank, options)
    }

    /// The name for the dual of another polytope.
    fn dual<T: NameType>(base: &Name<T>, options: Options) -> String {
        format!("{} dual", Self::base(base, options))
    }

    fn unknown() -> String {
        String::from("desconocido")
    }
}
