// use crate::models::{Result, Error};
// use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub fg: Color,
    pub bg: Option<Color>,
    pub modifiers: StyleModifiers,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StyleModifiers {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone)]
pub struct StyledToken {
    pub text: String,
    pub style: Style,
}

/// SIMD-accelerated syntax highlighter
pub struct SyntaxHighlighter {
    theme: Theme,
    token_cache: lru::LruCache<LineId, Vec<StyledToken>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineId(pub usize);

#[derive(Debug, Clone)]
pub struct Theme {
    pub keyword: Style,
    pub string: Style,
    pub number: Style,
    pub comment: Style,
    pub function: Style,
    pub variable: Style,
    pub operator: Style,
    pub default: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            keyword: Style {
                fg: Color::rgb(198, 120, 221), // Purple
                bg: None,
                modifiers: StyleModifiers { bold: true, ..Default::default() },
            },
            string: Style {
                fg: Color::rgb(152, 195, 121), // Green
                bg: None,
                modifiers: Default::default(),
            },
            number: Style {
                fg: Color::rgb(209, 154, 102), // Orange
                bg: None,
                modifiers: Default::default(),
            },
            comment: Style {
                fg: Color::rgb(92, 99, 112), // Gray
                bg: None,
                modifiers: StyleModifiers { italic: true, ..Default::default() },
            },
            function: Style {
                fg: Color::rgb(97, 175, 239), // Blue
                bg: None,
                modifiers: Default::default(),
            },
            variable: Style {
                fg: Color::rgb(224, 108, 117), // Red
                bg: None,
                modifiers: Default::default(),
            },
            operator: Style {
                fg: Color::rgb(86, 182, 194), // Cyan
                bg: None,
                modifiers: Default::default(),
            },
            default: Style {
                fg: Color::rgb(171, 178, 191), // Light gray
                bg: None,
                modifiers: Default::default(),
            },
        }
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            token_cache: lru::LruCache::new(std::num::NonZeroUsize::new(1000).unwrap()),
        }
    }
    
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme,
            token_cache: lru::LruCache::new(std::num::NonZeroUsize::new(1000).unwrap()),
        }
    }
    
    /// Highlight a line using SIMD-accelerated token classification
    pub fn highlight_line(&mut self, line: &str, line_id: LineId) -> Vec<StyledToken> {
        // Check cache first
        if let Some(cached) = self.token_cache.get(&line_id) {
            return cached.clone();
        }
        
        // SIMD-accelerated highlighting
        let tokens = self.highlight_line_simd(line);
        
        // Cache the result
        self.token_cache.put(line_id, tokens.clone());
        
        tokens
    }
    
    #[cfg(feature = "playground")]
    fn highlight_line_simd(&self, line: &str) -> Vec<StyledToken> {
        use simdutf8::basic::from_utf8;
        
        // Validate UTF-8 using SIMD
        if from_utf8(line.as_bytes()).is_err() {
            return vec![StyledToken {
                text: line.to_string(),
                style: self.theme.default,
            }];
        }
        
        // Tokenize and classify
        let mut tokens = Vec::new();
        let mut chars = line.chars().peekable();
        let mut current_token = String::new();
        
        while let Some(ch) = chars.next() {
            match ch {
                // String literals
                '"' => {
                    current_token.push(ch);
                    while let Some(ch) = chars.next() {
                        current_token.push(ch);
                        if ch == '"' && !current_token.ends_with("\\\"") {
                            break;
                        }
                    }
                    tokens.push(StyledToken {
                        text: std::mem::take(&mut current_token),
                        style: self.theme.string,
                    });
                }
                
                // Comments
                '/' if chars.peek() == Some(&'/') => {
                    current_token.push(ch);
                    for ch in chars.by_ref() {
                        current_token.push(ch);
                    }
                    tokens.push(StyledToken {
                        text: std::mem::take(&mut current_token),
                        style: self.theme.comment,
                    });
                }
                
                // Numbers
                '0'..='9' => {
                    current_token.push(ch);
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '.' || next_ch == '_' {
                            current_token.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(StyledToken {
                        text: std::mem::take(&mut current_token),
                        style: self.theme.number,
                    });
                }
                
                // Identifiers and keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    current_token.push(ch);
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            current_token.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    
                    let style = if self.is_keyword(&current_token) {
                        self.theme.keyword
                    } else if self.is_function_call(&current_token, &mut chars) {
                        self.theme.function
                    } else {
                        self.theme.variable
                    };
                    
                    tokens.push(StyledToken {
                        text: std::mem::take(&mut current_token),
                        style,
                    });
                }
                
                // Operators
                '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '|' | '^' | '%' => {
                    current_token.push(ch);
                    // Check for multi-char operators
                    if let Some(&next_ch) = chars.peek() {
                        if matches!((ch, next_ch), 
                            ('=', '=') | ('!', '=') | ('<', '=') | ('>', '=') |
                            ('&', '&') | ('|', '|') | ('<', '<') | ('>', '>') |
                            ('+', '=') | ('-', '=') | ('*', '=') | ('/', '='))
                        {
                            current_token.push(chars.next().unwrap());
                        }
                    }
                    tokens.push(StyledToken {
                        text: std::mem::take(&mut current_token),
                        style: self.theme.operator,
                    });
                }
                
                // Whitespace and other characters
                _ => {
                    if !current_token.is_empty() {
                        tokens.push(StyledToken {
                            text: std::mem::take(&mut current_token),
                            style: self.theme.default,
                        });
                    }
                    tokens.push(StyledToken {
                        text: ch.to_string(),
                        style: self.theme.default,
                    });
                }
            }
        }
        
        // Handle any remaining token
        if !current_token.is_empty() {
            tokens.push(StyledToken {
                text: current_token,
                style: self.theme.default,
            });
        }
        
        tokens
    }
    
    #[cfg(not(feature = "playground"))]
    fn highlight_line_simd(&self, line: &str) -> Vec<StyledToken> {
        vec![StyledToken {
            text: line.to_string(),
            style: self.theme.default,
        }]
    }
    
    fn is_keyword(&self, word: &str) -> bool {
        matches!(word,
            "fn" | "let" | "mut" | "if" | "else" | "while" | "for" | "loop" |
            "match" | "return" | "break" | "continue" | "struct" | "enum" |
            "impl" | "trait" | "pub" | "mod" | "use" | "self" | "super" |
            "crate" | "const" | "static" | "type" | "where" | "async" |
            "await" | "move" | "ref" | "in" | "as" | "true" | "false"
        )
    }
    
    fn is_function_call(&self, _word: &str, chars: &mut std::iter::Peekable<std::str::Chars>) -> bool {
        // Check if followed by '('
        let mut _whitespace_count = 0;
        while let Some(&ch) = chars.peek() {
            if ch == '(' {
                return true;
            } else if ch.is_whitespace() {
                _whitespace_count += 1;
                chars.next();
            } else {
                break;
            }
        }
        false
    }
    
    /// Clear the token cache
    pub fn clear_cache(&mut self) {
        self.token_cache.clear();
    }
    
    /// Invalidate cache for specific lines
    pub fn invalidate_lines(&mut self, start_line: usize, end_line: usize) {
        for line in start_line..=end_line {
            self.token_cache.pop(&LineId(line));
        }
    }
}

/// SIMD-accelerated utilities for token classification
#[cfg(feature = "playground")]
pub mod simd_utils {
    /// Check if a byte slice contains only ASCII alphanumeric characters
    #[cfg(target_arch = "x86_64")]
    pub fn is_ascii_alphanumeric_simd(bytes: &[u8]) -> bool {
        use std::arch::x86_64::*;
        
        unsafe {
            let chunks = bytes.chunks_exact(16);
            let remainder = chunks.remainder();
            
            for chunk in chunks {
                let data = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
                
                // Check for ASCII range
                let ascii_mask = _mm_cmplt_epi8(data, _mm_set1_epi8(-128i8));
                if _mm_movemask_epi8(ascii_mask) != 0xFFFF {
                    return false;
                }
                
                // Check for alphanumeric
                // Implement comparisons using available intrinsics
                // For >= we use NOT(a < b), for <= we use NOT(a > b)
                let zero = _mm_set1_epi8(b'0' as i8);
                let nine = _mm_set1_epi8(b'9' as i8);
                let is_digit = _mm_and_si128(
                    _mm_xor_si128(_mm_cmplt_epi8(data, zero), _mm_set1_epi8(-1i8)),
                    _mm_xor_si128(_mm_cmpgt_epi8(data, nine), _mm_set1_epi8(-1i8))
                );
                
                let upper_a = _mm_set1_epi8(b'A' as i8);
                let upper_z = _mm_set1_epi8(b'Z' as i8);
                let is_upper = _mm_and_si128(
                    _mm_xor_si128(_mm_cmplt_epi8(data, upper_a), _mm_set1_epi8(-1i8)),
                    _mm_xor_si128(_mm_cmpgt_epi8(data, upper_z), _mm_set1_epi8(-1i8))
                );
                
                let lower_a = _mm_set1_epi8(b'a' as i8);
                let lower_z = _mm_set1_epi8(b'z' as i8);
                let is_lower = _mm_and_si128(
                    _mm_xor_si128(_mm_cmplt_epi8(data, lower_a), _mm_set1_epi8(-1i8)),
                    _mm_xor_si128(_mm_cmpgt_epi8(data, lower_z), _mm_set1_epi8(-1i8))
                );
                
                let is_underscore = _mm_cmpeq_epi8(data, _mm_set1_epi8(b'_' as i8));
                
                let is_valid = _mm_or_si128(
                    _mm_or_si128(is_digit, is_upper),
                    _mm_or_si128(is_lower, is_underscore)
                );
                
                if _mm_movemask_epi8(is_valid) != 0xFFFF {
                    return false;
                }
            }
            
            // Check remainder
            remainder.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_')
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    pub fn is_ascii_alphanumeric_simd(bytes: &[u8]) -> bool {
        bytes.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_')
    }
}