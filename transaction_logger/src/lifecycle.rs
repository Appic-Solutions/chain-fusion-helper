use crate::endpoints::InitArgs;
use crate::endpoints::UpgradeArg;
use crate::logs::INFO;
use crate::state::ChainId;
use crate::state::Minter;

use crate::state::mutate_state;
use crate::state::MinterKey;
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

        if let Some(new_mintres) = args.new_minters {
            log!(INFO, "[init]: adding new minters: {:?}", new_mintres);

            let minters_iter = new_mintres
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
                    update_minter_args.oprator,
                );

                log!(
                    INFO,
                    "[init]: updating minter {:?} with args: {:?}",
                    minter_key,
                    update_minter_args
                );
                mutate_state(|s| {
                    s.update_minter_fees(
                        &minter_key,
                        update_minter_args.evm_to_icp_fee,
                        update_minter_args.icp_to_evm_fee,
                    )
                });
            }
        }
    }
}
