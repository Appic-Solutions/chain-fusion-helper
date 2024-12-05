use crate::endpoints::UpgradeArg;
use crate::logs::INFO;
use crate::state::ChainId;
use crate::state::Minter;

use crate::state::mutate_state;
use crate::state::MinterKey;
use ic_canister_log::log;

pub fn post_upgrade(upgrade_arg: Option<UpgradeArg>) {
    if let Some(arg) = upgrade_arg {
        log!(INFO, "[upgrade]: upgrading logger with arg: {:?}", arg);

        if let Some(new_mintres) = arg.new_minters {
            log!(INFO, "[init]: adding new minters: {:?}", new_mintres);

            let mapped_minters = new_mintres
                .iter()
                .map(|minter| Minter::from_minter_args(&minter));
            for minter in mapped_minters {
                mutate_state(|s| s.record_minter(minter))
            }
        }

        if let Some(update_minters) = arg.update_minters {
            for update_minter_args in update_minters {
                let minter_key = MinterKey(
                    ChainId::from(update_minter_args.clone().chain_id),
                    update_minter_args.clone().oprator,
                );

                mutate_state(|s| match s.get_minter_mut(&minter_key) {
                    Some(minter) => {
                        log!(
                            INFO,
                            "[init]: updating minter {:?} with args: {:?}",
                            minter,
                            update_minter_args
                        );
                        minter.evm_to_icp_fee = update_minter_args.evm_to_icp_fee;
                        minter.icp_to_evm_fee = update_minter_args.icp_to_evm_fee;
                    }
                    None => {}
                })
            }
        }
    }
}
