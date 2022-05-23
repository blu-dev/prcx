use std::collections::HashMap;

use prc::{ParamKind, ParamStruct, ParamList, hash40::{Hash40, hash40}};
use crate::GetPTag;

use super::{
    Result,
    Error,
    ParamTag
};

fn find_instance_of(struc: &Vec<(Hash40, ParamKind)>, key: Hash40, count: usize) -> Option<&ParamKind> {
    let mut current = 0;
    for (key_, param) in struc.iter() {
        if key == *key_ {
            if current == count {
                return Some(param);
            } else {
                current += 1;
            }
        }
    }
    None
}


pub fn generate_diff_struct(source: &ParamStruct, result: &ParamStruct) -> Result<Option<ParamKind>> {
    let ParamStruct(source) = source;
    let ParamStruct(result) = result;

    let mut vec = vec![];
    let mut already_encountered = HashMap::new();
    let mut diffs_encountered = HashMap::new();
    for (key, param) in result {
        let count = if let Some(count) = already_encountered.get_mut(key) {
            *count += 1;
            *count
        } else {
            already_encountered.insert(*key, 0);
            0
        };

        if let Some(src_param) = find_instance_of(source, *key, count) {
            if let Some(diff) = generate_diff(src_param, param)? {
                if count == 0 {
                    diffs_encountered.insert(*key, 1usize);
                    vec.push((*key, diff));
                } else {
                    if let Some(diff_count) = diffs_encountered.get_mut(key) {
                        if count == *diff_count {
                            *diff_count += 1;
                            vec.push((*key, diff));
                        } else {
                            for x in *diff_count..count {
                                if let Some(param) = find_instance_of(source, *key, x) {
                                    vec.push((*key, param.clone()));
                                } else {
                                    return Err(Error::BadReturn);
                                }
                            }
                            *diff_count = count + 1;
                            vec.push((*key, diff));
                        }
                    } else {
                        for x in 0..count {
                            if let Some(param) = find_instance_of(source, *key, x) {
                                vec.push((*key, param.clone()));
                            } else {
                                return Err(Error::BadReturn);
                            }
                        }
                        diffs_encountered.insert(*key, count + 1);
                        vec.push((*key, diff));
                    }
                }
            }
        } else {
            vec.push((*key, param.clone()));
        }
    }
    if vec.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ParamKind::Struct(ParamStruct(vec))))
    }
}

pub fn generate_diff_list(source: &ParamList, result: &ParamList) -> Result<Option<ParamKind>> {
    let ParamList(source) = source;
    let ParamList(result) = result;
    if result.len() < source.len() {
        return Err(Error::ShortPatchList);
    }

    let mut list = vec![];

    let mut is_non_dummy = false;

    for (idx, param) in result.iter().enumerate() {
        if let Some(src_param) = source.get(idx) {
            if let Some(diff) = generate_diff(src_param, param)? {
                list.push(diff);
                is_non_dummy = true;
            } else {
                list.push(ParamKind::Hash(hash40("dummy")));
            }
        } else {
            list.push(param.clone());
            is_non_dummy = true;
        }
    }

    if list.is_empty() || !is_non_dummy {
        Ok(None)
    } else {
        Ok(Some(ParamKind::List(ParamList(list))))
    }
}

pub fn generate_diff(source: &ParamKind, result: &ParamKind) -> Result<Option<ParamKind>> {
    if source.get_tag() != result.get_tag() {
        return Err(Error::NotSamePType);
    }

    if source.get_tag() == ParamTag::Struct {
        let source = if let ParamKind::Struct(struc) = source {
            struc
        } else {
            unreachable!()
        };

        let result = if let ParamKind::Struct(struc) = result {
            struc
        } else {
            unreachable!()
        };

        generate_diff_struct(source, result)
    } else if source.get_tag() == ParamTag::List {
        let source = if let ParamKind::List(list) = source {
            list
        } else {
            unreachable!()
        };

        let result = if let ParamKind::List(list) = result {
            list
        } else {
            unreachable!()
        };

        generate_diff_list(source, result)
    } else if source != result {
        Ok(Some(result.clone()))
    } else {
        Ok(None)
    }
}