//! Types for parsing the output of `malloc_info` from glibc.
//!
//! A best effort was made to account for all edge cases in the XML output of `malloc_info`, but
//! there may be some cases that are not accounted for. If you find one, please open an issue.

use serde::Deserialize;

/// Types of arena space
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AspaceType {
    Total,
    Mprotect,
    Subheaps,
    #[serde(other)]
    Other,
}

/// Arena space information
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Aspace {
    #[serde(rename = "@type")]
    pub r#type: AspaceType,
    #[serde(rename = "@size")]
    pub size: usize,
}

/// Types of system memory
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SystemType {
    Current,
    Max,
    #[serde(other)]
    Other,
}

/// System memory information
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct System {
    #[serde(rename = "@type")]
    pub r#type: SystemType,
    #[serde(rename = "@size")]
    pub size: usize,
}

/// Types of total memory
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TotalType {
    Fast,
    Rest,
    Mmap,
    #[serde(other)]
    Other,
}

/// Total memory information
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Total {
    #[serde(rename = "@type")]
    pub r#type: TotalType,
    #[serde(rename = "@count")]
    pub count: usize,
    #[serde(rename = "@size")]
    pub size: usize,
}

/// Size information for an arena or the whole heap
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Size {
    Size {
        #[serde(rename = "@from")]
        from: usize,
        #[serde(rename = "@to")]
        to: usize,
        #[serde(rename = "@total")]
        total: usize,
        #[serde(rename = "@count")]
        count: usize,
    },
    Unsorted {
        #[serde(rename = "@from")]
        from: usize,
        #[serde(rename = "@to")]
        to: usize,
        #[serde(rename = "@total")]
        total: usize,
        #[serde(rename = "@count")]
        count: usize,
    },
}

/// Wrapper type for sizes, which may be an array of XML elements
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Sizes {
    #[serde(rename = "$value")]
    pub sizes: Option<Vec<Size>>,
}

/// Arena-specific heap information
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Heap {
    /// Arena number
    #[serde(rename = "@nr")]
    pub nr: usize,

    /// Arena sizes
    pub sizes: Option<Sizes>,
}

/// Top-level type for all stats returned from [`malloc_info`](crate::malloc_info)
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Malloc {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "heap")]
    pub heaps: Vec<Heap>,
    pub total: Vec<Total>,
    pub system: Vec<System>,
    pub aspace: Vec<Aspace>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_simple() {
        // Taken from the malloc_info(3) man-page
        const XML: &str = r#"
<malloc version="1">
<heap nr="0">
<sizes>
</sizes>
<total type="fast" count="0" size="0"/>
<total type="rest" count="0" size="0"/>
<system type="current" size="135168"/>
<system type="max" size="135168"/>
<aspace type="total" size="135168"/>
<aspace type="mprotect" size="135168"/>
</heap>
<total type="fast" count="0" size="0"/>
<total type="rest" count="0" size="0"/>
<system type="current" size="135168"/>
<system type="max" size="135168"/>
<aspace type="total" size="135168"/>
<aspace type="mprotect" size="135168"/>
</malloc>
"#;
        let parsed: Malloc = quick_xml::de::from_str(XML).expect("parse XML");
        assert_eq!(parsed.version, "1");
        assert_eq!(parsed.heaps.len(), 1);
        assert_eq!(parsed.total.len(), 2);
        assert_eq!(parsed.system.len(), 2);
        assert_eq!(parsed.aspace.len(), 2);
    }

    #[test]
    fn parse_complex() {
        // Taken from the malloc_info(3) man-page
        const XML: &str = r#"
<malloc version="1">
<heap nr="0">
<sizes>
</sizes>
<total type="fast" count="0" size="0"/>
<total type="rest" count="0" size="0"/>
<system type="current" size="1081344"/>
<system type="max" size="1081344"/>
<aspace type="total" size="1081344"/>
<aspace type="mprotect" size="1081344"/>
</heap>
<heap nr="1">
<sizes>
</sizes>
<total type="fast" count="0" size="0"/>
<total type="rest" count="0" size="0"/>
<system type="current" size="1032192"/>
<system type="max" size="1032192"/>
<aspace type="total" size="1032192"/>
<aspace type="mprotect" size="1032192"/>
</heap>
<total type="fast" count="0" size="0"/>
<total type="rest" count="0" size="0"/>
<system type="current" size="2113536"/>
<system type="max" size="2113536"/>
<aspace type="total" size="2113536"/>
<aspace type="mprotect" size="2113536"/>
</malloc>
"#;
        let parsed: Malloc = quick_xml::de::from_str(XML).expect("parse XML");
        assert_eq!(parsed.version, "1");
        assert_eq!(parsed.heaps.len(), 2);
        assert_eq!(parsed.total.len(), 2);
        assert_eq!(parsed.system.len(), 2);
        assert_eq!(parsed.aspace.len(), 2);
    }

    #[test]
    #[should_panic]
    fn parse_invalid() {
        const XML: &str = r#"
<malloc version="1">
</malloc>
"#;
        let _ = quick_xml::de::from_str::<Malloc>(XML).expect("parse XML");
    }
}
