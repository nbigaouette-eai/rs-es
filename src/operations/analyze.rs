/*
 * Copyright 2015 Ben Ashford
 * Copyright 2015 Astro
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Implementation of ElasticSearch Analyze operation

use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};

use ::Client;
use ::error::EsError;

pub struct AnalyzeOperation<'a, 'b> {
    /// The HTTP client that this operation will use
    client:   &'a mut Client,

    index:    Option<&'b str>,
    req:      AnalyzeRequest<'b>
}

struct AnalyzeRequest<'b> {
    body:     &'b str,
    analyzer: Option<&'b str>
}

impl<'b> ToJson for AnalyzeRequest<'b> {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();

        d.insert("body".to_owned(), self.body.to_json());
        optional_add!(d, self.analyzer, "analyzer");

        Json::Object(d)
    }
}

impl<'a, 'b> AnalyzeOperation<'a, 'b> {
    pub fn new(client: &'a mut Client, body: &'b str) -> AnalyzeOperation<'a, 'b> {
        AnalyzeOperation {
            client:   client,
            index:    None,
            req: AnalyzeRequest {
                body:     body,
                analyzer: None
            }
        }
    }

    pub fn with_index(&'b mut self, index: &'b str) -> &'b mut Self {
        self.index = Some(index);
        self
    }

    pub fn with_analyzer(&'b mut self, analyzer: &'b str) -> &'b mut Self {
        self.req.analyzer = Some(analyzer);
        self
    }

    pub fn send(&'b mut self) -> Result<AnalyzeResult, EsError> {
        let url = match self.index {
            None => "/_analyze".to_owned(),
            Some(index) => format!("/{}/_analyze", index)
        };
        let (_, result) = try!(self.client.post_body_op(&url, &self.req.to_json()));
        Ok(AnalyzeResult::from(&result.expect("No Json payload")))
    }
}

/// The result of an analyze operation
#[derive(Debug)]
pub struct AnalyzeResult {
    pub tokens: Vec<Token>
}

#[derive(Debug)]
pub struct Token {
    pub token: String,
    pub token_type: String,
    pub position: u64,
    pub start_offset: u64,
    pub end_offset: u64
}

impl<'a> From<&'a Json> for AnalyzeResult {
    fn from(r: &'a Json) -> AnalyzeResult {
        let mut tokens = Vec::new();
        for t in get_json_array!(r, "tokens") {
            tokens.push(Token {
                token: get_json_string!(t, "token"),
                token_type: get_json_string!(t, "type"),
                position: get_json_u64!(t, "position"),
                start_offset: get_json_u64!(t, "start_offset"),
                end_offset: get_json_u64!(t, "end_offset")
            })
        }
        AnalyzeResult {
            tokens: tokens
        }
    }
}
