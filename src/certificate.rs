use std::{
    
    fmt::{

        Formatter as FmtFormatter,
        Result as FmtResult,
        Debug as FmtDebug,
    },

    net::{IpAddr},
};

use chrono::{
    
    NaiveDateTime,
    DateTime, 
    Utc,
};

use deepsize::{DeepSizeOf};

use serde::{

    Deserialize, 
    Serialize,
};

use x509_parser::prelude::{

    X509Certificate, 
    TbsCertificate, 
    GeneralName, 
    FromDer,
};

#[derive(Clone, Copy, Debug, DeepSizeOf)]
#[derive(Serialize, Deserialize)]
pub struct CertificateValidity {
    begin: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl<'a> CertificateValidity {
    pub(crate) fn from_timestamps(begin: i64, end: i64) -> CertificateValidity {

        CertificateValidity { 

            begin: DateTime::from_utc(NaiveDateTime::from_timestamp(begin.min(end), 0), Utc), 
            end: DateTime::from_utc(NaiveDateTime::from_timestamp(end.max(begin), 0), Utc), 
        }
    }

    pub fn is_within_valid_time(&'a self) -> bool {
        let now = Utc::now();

        if self.end > now {
            if self.begin < now {
                return true
            }
        }

        false
    }

    pub fn timestamp_begin(&'a self) -> i64 {
        self.begin.timestamp()
    }

    pub fn time_begin(&'a self) -> DateTime<Utc> {
        self.begin.clone()
    }

    pub fn timestamp_end(&'a self) -> i64 {
        self.end.timestamp()
    }

    pub fn time_end(&'a self) -> DateTime<Utc> {
        self.end.clone()
    }
}

#[derive(Clone, Debug, DeepSizeOf)]
#[derive(Serialize, Deserialize)]
pub enum CertificateAlternateName {
    Directory(String),
    Hostname(String),
    Address(String),
    Email(String),
    Uri(String),
}

impl<'a> CertificateAlternateName {
    pub fn to_string(&'a self) -> String{
        match self {
            CertificateAlternateName::Directory(ref string) => string.clone(),
            CertificateAlternateName::Hostname(ref string) => string.clone(),
            CertificateAlternateName::Address(ref string) => string.clone(),
            CertificateAlternateName::Email(ref string) => string.clone(),
            CertificateAlternateName::Uri(ref string) => string.clone(),
        }
    }

    pub fn as_str(&'a self) -> &'a str {
        match self {
            CertificateAlternateName::Directory(ref string) => string.as_str(),
            CertificateAlternateName::Hostname(ref string) => string.as_str(),
            CertificateAlternateName::Address(ref string) => string.as_str(),
            CertificateAlternateName::Email(ref string) => string.as_str(),
            CertificateAlternateName::Uri(ref string) => string.as_str(),
        }
    }
}

#[derive(Clone, DeepSizeOf)]
#[derive(Serialize, Deserialize)]
pub struct Certificate {
    pub(crate) issuer: Option<String>,
    pub(crate) authority: bool,
    pub(crate) organization: Option<String>,
    pub(crate) subject_name: Option<String>,
    pub(crate) subject_alternate: Vec<CertificateAlternateName>,
    pub(crate) validity: CertificateValidity,
    pub(crate) encoded: Vec<u8>,
}

impl<'a> Certificate {
    pub fn issuer(&'a self) -> Option<&'a str> {
        if let Some(ref issuer) = self.issuer {
            return Some(issuer.as_str())
        }

        None
    }

    pub fn authority(&'a self) -> bool {
        self.authority
    }

    pub fn organization(&'a self) -> Option<&'a str> {
        if let Some(ref organization) = self.organization {
            return Some(organization.as_str())
        }

        None
    }

    pub fn subject_name(&'a self) -> Option<&'a str> {
        if let Some(ref subject_name) = self.subject_name {
            return Some(subject_name.as_str())
        }

        None
    }

    pub fn subject_alternate(&'a self) -> &'a [CertificateAlternateName] {
        self.subject_alternate.as_slice()
    }

    pub fn validity(&'a self) -> CertificateValidity {
        self.validity
    }

    pub fn deep_size(&'a self) -> usize {
        self.deep_size_of()
    }

    pub fn encoded(&'a self) -> &'a [u8] {
        self.encoded.as_slice()
    }
}

impl FmtDebug for Certificate {
    fn fmt(&self, formatter: &mut FmtFormatter<'_>) -> FmtResult {
        formatter.debug_struct("Certificate")
            .field("issuer", &(self.issuer()))
            .field("authority", &(self.authority()))
            .field("organization", &(self.organization()))
            .field("subject_name", &(self.subject_name()))
            .field("subject_alternate", &(self.subject_alternate()))
            .field("validity", &(self.validity()))
            .finish()
    }
}

pub(crate) fn parse_certificate<'a>(data: &'a [u8]) -> Option<Certificate> {
    let (remaining, certificate) = if let Ok((remaining, certificate)) = X509Certificate::from_der(data) { 
        (remaining, certificate.tbs_certificate) 
    } 
    
    else {

        if let Ok((remaining, tbs_certificate)) = TbsCertificate::from_der(data) { 
            (remaining, tbs_certificate) 
        }
        
        else {
            
            return None
        }
    };

    let authority = certificate.is_ca();

    let general_names = if let Ok(Some(extension)) = certificate.subject_alternative_name() {
        extension.value.general_names.as_slice()
    } else { Default::default() };

    let issuer = certificate.issuer().iter_organization()
        .filter_map(|name| name.as_str().ok())
        .next().and_then(|name| Some({
            Some(name.to_owned())
        })).unwrap_or(None);

    let organization = certificate.subject().iter_organization()
        .filter_map(|name| name.as_str().ok())
        .next().and_then(|name| Some({
            Some(name.to_owned())
        })).unwrap_or(None);
    
    let subject_name = certificate.subject().iter_common_name()
        .filter_map(|name| name.as_str().ok())
        .next().and_then(|name| Some({
            Some(name.to_owned())
        })).unwrap_or(None);

    let subject_alternate = {
        general_names.iter().filter_map(|name| match name {
            GeneralName::DirectoryName(name) => Some({
                CertificateAlternateName::Directory(name.to_string())
            }),
            GeneralName::RFC822Name(name) => Some({
                CertificateAlternateName::Email(name.to_string())
            }),
            GeneralName::IPAddress(octets) => Some({
                CertificateAlternateName::Address(match octets {
                    octets if octets.len() == 4 => {
                        let mut array: [u8; 4] = Default::default();
                        for i in 0..4 { array[i] = octets[i] }
                        
                        IpAddr::from(array).to_string()
                    },
                    octets if octets.len() == 16 => {
                        let mut array: [u8; 16] = Default::default();
                        for i in 0..16 { array[i] = octets[i] }
                        
                        IpAddr::from(array).to_string()
                    },
                    _ => return None
                })
            }),
            GeneralName::DNSName(name) => Some({
                CertificateAlternateName::Hostname(name.to_string())
            }),
            GeneralName::URI(name) => Some({
                CertificateAlternateName::Uri(name.to_string())
            }),
            _ => None,
        }).filter_map(|alternate| match alternate.clone() {
            CertificateAlternateName::Directory(ref item) |
            CertificateAlternateName::Hostname(ref item) |
            CertificateAlternateName::Address(ref item) |
            CertificateAlternateName::Email(ref item) |
            CertificateAlternateName::Uri(ref item) => {
                if let Some(ref subject) = subject_name {
                    if item == subject.as_str() {
                        return None
                    }
                }

                Some(alternate)
            }
        }).collect()
    };

    let validity = {

        let begin = certificate.validity.not_before.timestamp();
        let end = certificate.validity.not_after.timestamp();

        CertificateValidity::from_timestamps(begin, end)
    };

    let encoded = data[..(data.len() - remaining.len())].to_vec();

    Some(Certificate {

        issuer, 
        authority,
        organization,
        subject_name,
        subject_alternate,
        validity,
        encoded,
    })
}