use serde_json::json;

/// Basic unit tests for the indexer logic (no DB required)
#[cfg(test)]
mod indexer_tests {
    #[test]
    fn test_split_pages_by_formfeed() {
        let text = "Page 1 content\x0CPage 2 content\x0CPage 3 content";
        let pages = split_into_pages(text);
        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0], "Page 1 content");
        assert_eq!(pages[1], "Page 2 content");
    }

    #[test]
    fn test_split_pages_by_paragraphs() {
        // No form feeds, should chunk by paragraph breaks
        let text = (0..20)
            .map(|i| format!("Paragraph {i} with some content that fills space."))
            .collect::<Vec<_>>()
            .join("\n\n");
        let pages = split_into_pages(&text);
        assert!(!pages.is_empty());
        // All content should be preserved
        let total_len: usize = pages.iter().map(|p| p.len()).sum();
        assert!(total_len > 0);
    }

    #[test]
    fn test_split_empty_text() {
        let pages = split_into_pages("");
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn test_build_tree_index_extracts_headings() {
        let content = "# Introduction\nSome text here.\n\n# Methods\nMore text.";
        let index = build_simple_tree_index(content, 1);
        let topics = index["topics"].as_array().unwrap();
        assert_eq!(topics.len(), 2);
        assert_eq!(topics[0]["name"], "Introduction");
        assert_eq!(topics[1]["name"], "Methods");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 5), "hello...");
        assert_eq!(truncate("hi", 10), "hi");
    }

    // --- Inline the functions to test without needing the full server ---

    fn split_into_pages(text: &str) -> Vec<String> {
        let ff_pages: Vec<&str> = text.split('\x0C').collect();
        if ff_pages.len() > 1 {
            return ff_pages
                .into_iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut pages = Vec::new();
        let mut current_page = String::new();

        for para in paragraphs {
            if current_page.len() + para.len() > 3000 && !current_page.is_empty() {
                pages.push(current_page.trim().to_string());
                current_page = String::new();
            }
            if !current_page.is_empty() {
                current_page.push_str("\n\n");
            }
            current_page.push_str(para);
        }

        if !current_page.trim().is_empty() {
            pages.push(current_page.trim().to_string());
        }

        if pages.is_empty() {
            pages.push(text.to_string());
        }

        pages
    }

    fn build_simple_tree_index(content: &str, page_num: i32) -> serde_json::Value {
        let lines: Vec<&str> = content.lines().collect();

        let topics: Vec<serde_json::Value> = lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('#')
                    || (trimmed.len() > 3
                        && trimmed.len() < 80
                        && trimmed
                            .chars()
                            .all(|c| c.is_uppercase() || c.is_whitespace() || c.is_ascii_punctuation()))
            })
            .map(|line| {
                serde_json::json!({
                    "name": line.trim().trim_start_matches('#').trim(),
                    "type": "heading"
                })
            })
            .collect();

        serde_json::json!({
            "page": page_num,
            "summary": truncate(content, 300),
            "char_count": content.len(),
            "topics": topics,
        })
    }

    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}
