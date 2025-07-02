use crate::endpoints::InitArgs;
use crate::endpoints::UpgradeArg;
use crate::logs::INFO;
use crate::state::nat_to_erc20_amount;
use crate::state::nat_to_u64;
use crate::state::types::{ChainId, Minter, MinterKey};

use crate::state::mutate_state;
use ic_canister_log::log;

pub fn init(init_args: InitArgs) {
    let minters_iter = init_args
        .minters
        .into_iter()
        .map(|minter_arg| Minter::from_minter_args(minter_arg));

    for minter in minters_iter {
        mutate_state(|s| s.record_minter(minter));
    }
}

pub fn post_upgrade(upgrade_arg: Option<UpgradeArg>) {
    if let Some(args) = upgrade_arg {
        log!(INFO, "[upgrade]: upgrading logger with arg: {:?}", args);

        if let Some(new_minters) = args.new_minters {
            log!(INFO, "[init]: adding new minters: {:?}", new_minters);

            let minters_iter = new_minters
                .into_iter()
                .map(|minter| Minter::from_minter_args(minter));
            for minter in minters_iter {
                mutate_state(|s| s.record_minter(minter))
            }
        }

        if let Some(update_minters) = args.update_minters {
            for update_minter_args in update_minters {
                let minter_key = MinterKey(
                    ChainId::from(&update_minter_args.chain_id),
                    update_minter_args.operator,
                );

                log!(
                    INFO,
                    "[init]: updating minter {:?} with args: {:?}",
                    minter_key,
                    update_minter_args
                );
                if let Some(last_observed) = &update_minter_args.last_observed_event {
                    mutate_state(|s| {
                        s.update_last_observed_event(&minter_key, nat_to_u64(last_observed))
                    })
                }
                if let Some(last_scraped) = &update_minter_args.last_observed_event {
                    mutate_state(|s| {
                        s.update_last_scraped_event(&minter_key, nat_to_u64(last_scraped))
                    })
                }
            }
        }
    }
}
