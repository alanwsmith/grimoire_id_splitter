#![allow(unused)]
use nom::branch::alt;
use nom::bytes::complete::is_a;
use nom::bytes::complete::tag;
use nom::bytes::complete::tag_no_case;
use nom::bytes::complete::take_until;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::anychar;
use nom::character::complete::line_ending;
use nom::character::complete::multispace1;
use nom::character::complete::satisfy;
use nom::character::complete::space0;
use nom::character::complete::space1;
use nom::character::is_alphanumeric;
use nom::combinator::eof;
use nom::combinator::not;
use nom::combinator::opt;
use nom::combinator::rest;
use nom::multi::many0;
use nom::multi::many1;
use nom::multi::many_m_n;
use nom::multi::many_till;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;
use nom::Parser;
use std::fs;
use std::path::PathBuf;

fn get_single_non_whitespace_character(source: &str) -> IResult<&str, char> {
    let (source, captured) = preceded(not(multispace1), anychar)(source)?;
    Ok((source, captured))
}

fn get_to_last_instance_of_page<'a>(source: &'a str) -> IResult<&'a str, String> {
    let (source, captured) = many1(get_to_page)(source)?;
    Ok((source, captured.join("")))
}

fn get_to_page<'a>(source: &'a str) -> IResult<&'a str, String> {
    let (source, captured) = many0(
        tuple((take_until("--"), is_a("-"), space0, not(tag("page"))))
            .map(|hit| format!("{}{}{}", hit.0, hit.1, hit.2)),
    )(source)?;
    let (source, captured2) = tuple((
        take_until("--"),
        is_a("-"),
        space1,
        tag("page"),
        space0,
        line_ending,
    ))
    .map(|hit| format!("{}{}{}{}{}{}", hit.0, hit.1, hit.2, hit.3, hit.4, hit.5))
    .parse(source)?;
    let mut result = captured.join("");
    result.push_str(&captured2);
    Ok((source, result))
}

fn get_to_last_instance_of_id<'a>(source: &'a str) -> IResult<&'a str, String> {
    let (source, captured) = many1(get_to_id)(source)?;
    Ok((source, captured.join("")))
}

fn get_to_id<'a>(source: &'a str) -> IResult<&'a str, String> {
    let (source, captured) = many0(
        tuple((take_until("--"), is_a("-"), space0, not(tag("id:"))))
            .map(|hit| format!("{}{}{}", hit.0, hit.1, hit.2)),
    )(source)?;
    let (source, captured2) = tuple((take_until("--"), is_a("-"), space1, tag("id:"), space1))
        .map(|hit| format!("{}{}{}{}{}", hit.0, hit.1, hit.2, hit.3, hit.4))
        .parse(source)?;
    let mut result = captured.join("");
    result.push_str(&captured2);
    Ok((source, result))
}

fn get_id_with_update(source: &str) -> IResult<&str, String> {
    let (source, captured) = opt(terminated(
        many_m_n(
            4,
            4,
            pair(
                get_single_non_whitespace_character,
                get_single_non_whitespace_character,
            )
            .map(|x| format!("{}{}", x.0, x.1)),
        ),
        alt((multispace1, eof)),
    ))(source)?;
    match captured {
        Some(parts) => Ok((source, format!("{}\n", parts.join("/")))),
        None => Ok((source, "".to_string())),
    }
}

fn get_updated_source(source: &str) -> IResult<&str, String> {
    let (source, to_page) = get_to_last_instance_of_page(source)?;
    let (source, to_id) = get_to_last_instance_of_id(source)?;
    let (source, updated_id) = get_id_with_update(source)?;
    let (source, tail) = rest(source)?;
    Ok((
        source,
        format!("{}{}{}{}", to_page, to_id, updated_id, tail),
    ))
}

pub fn get_files_with_extension_in_a_single_directory(
    dir: &PathBuf,
    extension: &str,
) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap()
        .into_iter()
        .filter(|p| {
            if p.as_ref().unwrap().path().is_file() {
                true
            } else {
                false
            }
        })
        .filter(|p| match p.as_ref().unwrap().path().extension() {
            Some(ext) => ext == extension,
            None => false,
        })
        .filter_map(|p| match p.as_ref().unwrap().path().strip_prefix(".") {
            Ok(_) => None,
            Err(_) => Some(p.as_ref().unwrap().path()),
        })
        .collect()
}

fn main() {
    let file_list = get_files_with_extension_in_a_single_directory(
        &PathBuf::from("/Users/alan/Grimoire"),
        "neo",
    );
    for f in file_list.iter() {
        match f.file_name() {
            Some(file_name) => {
                let output_path =
                    PathBuf::from("/Users/alan/Desktop/grimoire_test").join(file_name);
                match fs::read_to_string(f) {
                    Ok(source) => {
                        match get_updated_source(&source) {
                            Ok((remainder, output)) => {
                                fs::write(f, output).unwrap();
                                //dbg!("here");
                                ()
                            }
                            Err(e) => {
                                // fs::write(f, source).unwrap();
                                //dbg!(&file_name);
                                //dbg!(e);
                                ()
                            }
                        }
                        // dbg!("got it");
                        ()
                    }
                    Err(e) => {
                        dbg!(e);
                        ()
                    }
                }
                //dbg!(file_name);
                //dbg!(output_path);
                ()
            }
            None => {
                dbg!("x");
                ()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case(
        "something\n\n-- page\n-- status: whatever\n-- id: abcd1234\n-- type: whatever",
        "something\n\n-- page\n-- status: whatever\n-- id: ab/cd/12/34\n-- type: whatever".to_string()
    )]
    #[case(
        "something\n\n-- page\n-- status: whatever\n-- id: abcd1234",
        "something\n\n-- page\n-- status: whatever\n-- id: ab/cd/12/34\n".to_string()
    )]
    #[case(
        "something\n\n-- page\n-- status: whatever\n-- id: abcd1234x\n-- type: whatever",
        "something\n\n-- page\n-- status: whatever\n-- id: abcd1234x\n-- type: whatever".to_string()
    )]
    #[case(
        "something\n\n-- page\n-- status: whatever\n-- id: abcd123\n-- type: whatever",
        "something\n\n-- page\n-- status: whatever\n-- id: abcd123\n-- type: whatever".to_string()
    )]
    #[case(
        "something\n\n-- page\n-- status: whatever\n-- id: abcd1234\n-- type: whatever\n\n-- page\n-- id: qwer7890\n",
        "something\n\n-- page\n-- status: whatever\n-- id: abcd1234\n-- type: whatever\n\n-- page\n-- id: qw/er/78/90\n".to_string()
    )]
    #[case(
        "----something\n\n-- page\n-- status: whatever\n-- id: abcd1234",
        "----something\n\n-- page\n-- status: whatever\n-- id: ab/cd/12/34\n".to_string()
    )]
    fn get_updated_source_test(#[case] source: &str, #[case] left: String) {
        let right = get_updated_source(source);
        assert_eq!(
            "",
            right.as_ref().unwrap().0,
            "Make sure remainder is empty"
        );
        assert_eq!(left, right.unwrap().1, "Confirm output");
    }
}
