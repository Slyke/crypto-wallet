use crate::{
    bitcoin::{self, Inspection},
    entropy,
    model::{self, Account, History, Identity, Mode, Root, Settings, VaultData},
    protocol::{PublicBundle, Request, Response},
    storage::{self, OpenVault},
    Error, Result,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use uuid::Uuid;
use zeroize::Zeroizing;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub initialized: bool,
    pub unlocked: bool,
    pub network_build: bool,
    pub settings: Option<Settings>,
    pub roots: Vec<Root>,
    pub identities: Vec<Identity>,
    pub accounts: Vec<Account>,
    pub history: Vec<History>,
    pub pending: Vec<Request>,
    pub responses: Vec<Response>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ceremony {
    pub token: Uuid,
    pub phrase: String,
}
struct PendingCeremony {
    token: Uuid,
    phrase: Zeroizing<String>,
    at: Instant,
}
struct Approval {
    token: Uuid,
    request_id: Uuid,
    digest: String,
    at: Instant,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub token: Uuid,
    pub inspection: Inspection,
}
pub struct Session {
    path: PathBuf,
    vault: Option<OpenVault>,
    ceremony: Option<PendingCeremony>,
    approval: Option<Approval>,
    last_activity: Instant,
    last_tick: Instant,
}
impl Session {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            vault: None,
            ceremony: None,
            approval: None,
            last_activity: Instant::now(),
            last_tick: Instant::now(),
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        let data = self.vault.as_ref().map(|v| &v.data);
        Snapshot {
            initialized: self.path.exists(),
            unlocked: data.is_some(),
            network_build: cfg!(feature = "network"),
            settings: data.map(|d| d.settings.clone()),
            roots: data.map(|d| d.roots.clone()).unwrap_or_default(),
            identities: data.map(|d| d.identities.clone()).unwrap_or_default(),
            accounts: data.map(|d| d.accounts.clone()).unwrap_or_default(),
            history: data.map(|d| d.history.clone()).unwrap_or_default(),
            pending: data.map(|d| d.pending.clone()).unwrap_or_default(),
            responses: data.map(|d| d.responses.clone()).unwrap_or_default(),
        }
    }
    pub fn create(&mut self, password: &str, mode: Mode) -> Result<()> {
        if self.path.exists() {
            return Err(Error::Invalid(
                "a vault already exists; restore only on a fresh profile",
            ));
        }
        if !cfg!(feature = "network") && mode != Mode::Offline {
            return Err(Error::Mode);
        }
        let mut data = VaultData::default();
        data.settings.mode = mode;
        let mut vault = OpenVault::create(password, data)?;
        vault.save(&self.path)?;
        self.vault = Some(vault);
        self.touch();
        Ok(())
    }
    pub fn unlock(&mut self, password: &str) -> Result<()> {
        self.lock();
        let vault = OpenVault::unlock(
            &storage::read_bounded(&self.path, storage::MAX_VAULT_BYTES)?,
            password,
        )?;
        if !cfg!(feature = "network") && vault.data.settings.mode != Mode::Offline {
            return Err(Error::Mode);
        }
        if vault.data.settings.mode == Mode::Proxy && !vault.data.records.is_empty() {
            return Err(Error::Mode);
        }
        self.vault = Some(vault);
        self.touch();
        Ok(())
    }
    pub fn cancel_ceremony(&mut self) {
        self.ceremony = None;
    }
    pub fn lock(&mut self) {
        self.ceremony = None;
        self.approval = None;
        self.vault = None;
    }
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }
    pub fn tick(&mut self) {
        let missed = self.last_tick.elapsed() > Duration::from_secs(15);
        self.last_tick = Instant::now();
        if self.vault.as_ref().is_some_and(|v| {
            self.last_activity.elapsed().as_secs() >= v.data.settings.timeout_seconds
        }) || missed
        {
            self.lock();
        }
        if self
            .ceremony
            .as_ref()
            .is_some_and(|c| c.at.elapsed() > Duration::from_secs(300))
        {
            self.ceremony = None;
        }
    }
    pub fn focus_lost(&mut self) {
        if self
            .vault
            .as_ref()
            .is_some_and(|v| v.data.settings.lock_on_blur)
        {
            self.lock();
        }
    }
    pub fn vault(&self) -> Result<&OpenVault> {
        self.vault.as_ref().ok_or(Error::Locked)
    }
    /// Roll back metadata and wrapping changes when an operation or durable save fails.
    pub fn mutate<T>(&mut self, operation: impl FnOnce(&mut OpenVault) -> Result<T>) -> Result<T> {
        let vault = self.vault.as_mut().ok_or(Error::Locked)?;
        let previous_data = vault.data.clone();
        let previous_file = vault.file.clone();
        let outcome = operation(vault).and_then(|value| {
            vault.save(&self.path)?;
            Ok(value)
        });
        if outcome.is_err() {
            vault.data = previous_data;
            vault.file = previous_file;
        }
        self.touch();
        outcome
    }
    pub fn generate(&mut self, words: u8, prefix: &str, bits: Option<&str>) -> Result<Ceremony> {
        if !self.vault()?.data.settings.mode.can_sign() {
            return Err(Error::Mode);
        }
        let phrase = entropy::generate(words, prefix, bits)?;
        let token = Uuid::new_v4();
        let ceremony = Ceremony {
            token,
            phrase: phrase.to_string(),
        };
        self.ceremony = Some(PendingCeremony {
            token,
            phrase,
            at: Instant::now(),
        });
        self.touch();
        Ok(ceremony)
    }
    pub fn confirm_phrase(
        &mut self,
        token: Uuid,
        confirmation: &str,
        label: &str,
        password: Option<&str>,
    ) -> Result<Uuid> {
        use subtle::ConstantTimeEq;
        let pending = self.ceremony.take().ok_or(Error::Approval)?;
        if pending.token != token || pending.at.elapsed() > Duration::from_secs(300) {
            return Err(Error::Approval);
        }
        let confirmation = entropy::validate(confirmation)?;
        if !bool::from(confirmation.as_bytes().ct_eq(pending.phrase.as_bytes())) {
            return Err(Error::Invalid(
                "recovery confirmation did not match; begin creation again",
            ));
        }
        self.mutate(|v| v.import_phrase(label, &pending.phrase, password))
    }
    pub fn import_public(&mut self, bytes: &[u8]) -> Result<()> {
        let bundle = PublicBundle::parse(bytes)?;
        self.mutate(|v| {
            for mut account in bundle.accounts {
                if v.data
                    .accounts
                    .iter()
                    .any(|a| a.network == account.network && a.address == account.address)
                {
                    continue;
                }
                account.id = Uuid::new_v4();
                v.data.accounts.push(account);
            }
            Ok(())
        })
    }
    pub fn public_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec_pretty(&PublicBundle::new(
            self.vault()?.public_accounts(),
        )?)?)
    }
    pub fn import_request(&mut self, bytes: &[u8]) -> Result<Uuid> {
        let request = Request::parse(bytes)?;
        let id = request.request_id;
        bitcoin::inspect(&request, &self.vault()?.data.accounts)?;
        self.mutate(|v| {
            if v.data.pending.iter().any(|r| r.request_id == id)
                || v.data
                    .history
                    .iter()
                    .any(|h| h.request_id == id && h.action == "signed")
            {
                return Err(Error::Duplicate);
            }
            if v.data.pending.len() >= 500 {
                return Err(Error::Invalid("pending request limit"));
            }
            v.data.pending.push(request);
            Ok(id)
        })
    }
    pub fn review(&mut self, id: Uuid) -> Result<Review> {
        let vault = self.vault()?;
        let request = vault
            .data
            .pending
            .iter()
            .find(|r| r.request_id == id)
            .ok_or(Error::NotFound)?;
        let inspection =
            if let Some(response) = vault.data.responses.iter().find(|r| r.request_id == id) {
                bitcoin::verify(request, response, &vault.data.accounts)?
            } else {
                bitcoin::inspect(request, &vault.data.accounts)?
            };
        let token = Uuid::new_v4();
        self.approval = Some(Approval {
            token,
            request_id: id,
            digest: inspection.digest.clone(),
            at: Instant::now(),
        });
        self.touch();
        Ok(Review { token, inspection })
    }
    pub fn sign(
        &mut self,
        token: Uuid,
        confirmed: bool,
        master_password: &str,
        key_passwords: &HashMap<Uuid, Zeroizing<String>>,
        selected_accounts: &[Uuid],
    ) -> Result<Response> {
        let approval = self.approval.take().ok_or(Error::Approval)?;
        if !confirmed || token != approval.token || approval.at.elapsed() > Duration::from_secs(120)
        {
            return Err(Error::Approval);
        }
        self.mutate(|vault| {
            if !vault.data.settings.mode.can_sign() {
                return Err(Error::Mode);
            }
            vault.authenticate(master_password)?;
            let request = vault
                .data
                .pending
                .iter()
                .find(|r| r.request_id == approval.request_id)
                .ok_or(Error::NotFound)?
                .clone();
            if request.binding()? != approval.digest {
                return Err(Error::Approval);
            }
            if vault
                .data
                .history
                .iter()
                .any(|h| h.request_id == request.request_id && h.action == "signed")
            {
                return Err(Error::Duplicate);
            }
            let inspection = bitcoin::inspect(&request, &vault.data.accounts)?;
            let mut keys = vec![];
            for input in &inspection.inputs {
                let matches: Vec<_> = vault
                    .data
                    .accounts
                    .iter()
                    .filter(|a| {
                        input.account_ids.contains(&a.id)
                            && !a.watch_only
                            && a.identity_id.is_some()
                    })
                    .collect();
                let account = if matches.len() == 1 {
                    matches[0]
                } else {
                    let selected: Vec<_> = matches
                        .iter()
                        .filter(|a| selected_accounts.contains(&a.id))
                        .collect();
                    if selected.len() != 1 {
                        return Err(Error::Invalid(
                            "select exactly one matching local account for each ambiguous input",
                        ));
                    }
                    *selected[0]
                };
                let identity_id = account.identity_id.ok_or(Error::NotFound)?;
                let identity = vault
                    .data
                    .identities
                    .iter()
                    .find(|i| i.id == identity_id)
                    .ok_or(Error::NotFound)?;
                let password = key_passwords.get(&identity.root_id).map(|p| p.as_str());
                let (key, _) =
                    vault.derive_secret(identity_id, account.network, &account.path, password)?;
                let (address, public) =
                    crate::wallet::public_account(&key, account.network, &account.path)?;
                if address != input.address || public != account.public_key {
                    return Err(Error::Verification);
                }
                keys.push(key);
            }
            let response = bitcoin::sign(&request, &vault.data.accounts, &keys)?;
            vault.data.responses.push(response.clone());
            vault.data.history.push(History {
                id: Uuid::new_v4(),
                request_id: request.request_id,
                action: "signed".into(),
                at: model::now(),
                digest: approval.digest,
                txid: Some(response.txid.clone()),
            });
            Ok(response)
        })
    }
    pub fn import_response(&mut self, bytes: &[u8]) -> Result<Inspection> {
        let response = Response::parse(bytes)?;
        self.mutate(|v| {
            let request = v
                .data
                .pending
                .iter()
                .find(|r| r.request_id == response.request_id)
                .ok_or(Error::NotFound)?;
            let inspection = bitcoin::verify(request, &response, &v.data.accounts)?;
            if v.data
                .responses
                .iter()
                .any(|r| r.request_id == response.request_id)
            {
                return Err(Error::Duplicate);
            }
            v.data.history.push(History {
                id: Uuid::new_v4(),
                request_id: request.request_id,
                action: "verified".into(),
                at: model::now(),
                digest: response.request_digest.clone(),
                txid: Some(response.txid.clone()),
            });
            v.data.responses.push(response);
            Ok(inspection)
        })
    }
    pub fn settings(&mut self, settings: Settings, password: &str, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Err(Error::Approval);
        }
        if !(30..=3600).contains(&settings.timeout_seconds) || settings.rpc_url.len() > 2048 {
            return Err(Error::Invalid("settings bounds"));
        }
        if !cfg!(feature = "network") && settings.mode != Mode::Offline {
            return Err(Error::Mode);
        }
        self.approval = None;
        self.mutate(|v| {
            v.authenticate(password)?;
            if settings.mode == Mode::Proxy && !v.data.records.is_empty() {
                return Err(Error::Invalid(
                    "proxy mode requires a vault containing only watch-only accounts",
                ));
            }
            v.data.settings = settings;
            Ok(())
        })
    }
    pub fn backup(&mut self, destination: &Path, password: &str, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Err(Error::Approval);
        }
        if destination == self.path {
            return Err(Error::Invalid("backup destination is the active vault"));
        }
        let vault = self.vault.as_mut().ok_or(Error::Locked)?;
        vault.authenticate(password)?;
        storage::atomic_write(destination, &vault.encrypted_bytes()?)
    }
    pub fn restore(&mut self, source: &Path, password: &str, dry_run: bool) -> Result<()> {
        let bytes = storage::read_bounded(source, storage::MAX_VAULT_BYTES)?;
        let vault = OpenVault::unlock(&bytes, password)?;
        if !dry_run {
            if self.path.exists() {
                return Err(Error::Invalid(
                    "restore requires a fresh application data directory",
                ));
            }
            if !cfg!(feature = "network") && vault.data.settings.mode != Mode::Offline {
                return Err(Error::Mode);
            }
            storage::atomic_write(&self.path, &bytes)?;
            self.lock();
        }
        Ok(())
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    #[test]
    fn expiry_cancellation_and_lock_invalidate_creation_ceremonies() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(dir.path().join("vault.avvault"));
        session
            .create("public-test-password", Mode::Offline)
            .unwrap();
        let ceremony = session.generate(12, "", Some(&"0".repeat(128))).unwrap();
        session.cancel_ceremony();
        assert!(session
            .confirm_phrase(ceremony.token, &ceremony.phrase, "Cancelled", None)
            .is_err());
        let ceremony = session.generate(12, "", Some(&"0".repeat(128))).unwrap();
        session.ceremony.as_mut().unwrap().at = Instant::now() - Duration::from_secs(301);
        session.tick();
        assert!(session
            .confirm_phrase(ceremony.token, &ceremony.phrase, "Expired", None)
            .is_err());
        session.last_activity = Instant::now() - Duration::from_secs(301);
        session.tick();
        assert!(!session.snapshot().unlocked);
        assert!(session.snapshot().roots.is_empty());
    }
    #[test]
    fn expired_approval_fails_before_key_or_request_lookup() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(dir.path().join("vault.avvault"));
        session
            .create("public-test-password", Mode::Offline)
            .unwrap();
        let token = Uuid::new_v4();
        session.approval = Some(Approval {
            token,
            request_id: Uuid::new_v4(),
            digest: String::new(),
            at: Instant::now() - Duration::from_secs(121),
        });
        assert!(matches!(
            session.sign(token, true, "public-test-password", &HashMap::new(), &[]),
            Err(Error::Approval)
        ));
        assert!(session.approval.is_none());
        session.last_tick = Instant::now() - Duration::from_secs(16);
        session.tick();
        assert!(!session.snapshot().unlocked);
    }
}
