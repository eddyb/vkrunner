// vkrunner
//
// Copyright (C) 2018 Intel Corporation
// Copyright 2023 Neil Roberts
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::vk;

/// An enum of all the possible shader stages known to VkRunner.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C)]
pub enum Stage {
    Vertex = 0,
    TessCtrl,
    TessEval,
    Geometry,
    Fragment,
    Compute
}

/// The number of shader stages known to VkRunner. This should match
/// the number of values in [Stage].
pub(crate) const N_STAGES: usize = 6;

/// All the possible stage values.
pub(crate) static ALL_STAGES: [Stage; N_STAGES] = [
    Stage::Vertex,
    Stage::TessCtrl,
    Stage::TessEval,
    Stage::Geometry,
    Stage::Fragment,
    Stage::Compute,
];

impl Stage {
    /// Get the corresponding flag for this stage. This can be used to
    /// store the stages in a bitmask.
    pub(crate) const fn flag(self) -> vk::VkShaderStageFlagBits {
        vk::VK_SHADER_STAGE_VERTEX_BIT << self as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flags() {
        assert_eq!(Stage::Vertex.flag(), vk::VK_SHADER_STAGE_VERTEX_BIT);
        assert_eq!(
            Stage::TessCtrl.flag(),
            vk::VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT,
        );
        assert_eq!(
            Stage::TessEval.flag(),
            vk::VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT,
        );
        assert_eq!(Stage::Geometry.flag(), vk::VK_SHADER_STAGE_GEOMETRY_BIT);
        assert_eq!(Stage::Fragment.flag(), vk::VK_SHADER_STAGE_FRAGMENT_BIT);
        assert_eq!(Stage::Compute.flag(), vk::VK_SHADER_STAGE_COMPUTE_BIT);
    }
}
