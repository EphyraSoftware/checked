use hdk::prelude::*;

pub struct PrepareFetchRequest {
    pub fetch_url: String,
}

#[hdk_extern]
fn prepare_fetch(request: PrepareFetchRequest) -> ExternResult<()> {
    url::Url::parse(&request.fetch_url)?;

    Ok(())
}
