use std::collections::HashMap;

use prc::{ParamKind, ParamStruct, ParamList, hash40::{Hash40, hash40}};

use crate::{GetPTag, ParamTag};

use super::{
    Result,
    Error
};

fn find_instance_of(struc: &mut Vec<(Hash40, ParamKind)>, key: Hash40, count: usize) -> Option<&mut ParamKind> {
    let mut current = 0;
    for (key_, param) in struc.iter_mut() {
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

pub fn apply_patch_struct(patch: &ParamStruct, source: &mut ParamStruct) -> Result<()> {
    let ParamStruct(patch) = patch;
    let ParamStruct(source) = source;

    let mut encountered_patches = HashMap::new();

    for (key, param) in patch.iter() {
        let count = if let Some(count) = encountered_patches.get_mut(key) {
            *count += 1;
            *count
        } else {
            encountered_patches.insert(*key, 0usize);
            0
        };

        if let Some(src_param) = find_instance_of(source, *key, count) {
            apply_patch(param, src_param)?;
        } else {
            source.push((*key, param.clone()));
        }
    }

    Ok(())
}

pub fn apply_patch_list(patch: &ParamList, source: &mut ParamList) -> Result<()> {
    let ParamList(patch) = patch;
    let ParamList(source) = source;

    /*if patch.len() < source.len() {
        return Err(Error::ShortPatchList);
    }*/

    for (idx, param) in patch.iter().enumerate() {
        if *param == ParamKind::Hash(hash40("dummy")) {
            continue;
        }

        if let Some(src_param) = source.get_mut(idx) {
            apply_patch(param, src_param)?;
        } else {
            source.push(param.clone());
        }
    }


    Ok(())
}

pub fn apply_patch(patch: &ParamKind, source: &mut ParamKind) -> Result<()> {
    if patch.get_tag() != source.get_tag() {
        return Err(Error::NotSamePType);
    }

    if patch.get_tag() == ParamTag::Struct {
        let patch = if let ParamKind::Struct(struc) = patch {
            struc
        } else {
            unreachable!()
        };

        let source = if let ParamKind::Struct(struc) = source {
            struc
        } else {
            unreachable!()
        };

        apply_patch_struct(patch, source)?;
    } else if patch.get_tag() == ParamTag::List {
        let patch = if let ParamKind::List(list) = patch {
            list
        } else {
            unreachable!()
        };

        let source = if let ParamKind::List(list) = source  {
            list
        } else {
            unreachable!()
        };

        apply_patch_list(patch, source)?;
    } else {
        *source = patch.clone();
    }

    Ok(())
}