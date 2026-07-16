// Copyright 2026 agwlvssainokuni
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `business-logic-model.md`のTestable Properties（PBT-01）に基づく
//! プロパティベーステストのエントリーポイント。

mod array_repeat;
mod directory_resolver_idempotence;
mod escape_roundtrip;
mod partial_recursion_guard;
mod section_complement;
mod section_nesting;
mod support;
mod text_passthrough;
