import { utils } from 'near-api-js';

export class TheContract {

  constructor({ contractId, walletToUse }) {
    this.contractId = contractId;
    this.wallet = walletToUse
  }

  async getMatches() {
    // View all the matches
    const matches = await this.wallet.viewMethod({ contractId: this.contractId, method: "view_matches", args: { match_id: "all"}})
    return matches
  }

  async getPotentialWinnings({ matchId, team, betAmount }) {
    // Views potential winnings
    console.log(matchId)
    const potentialWinnings = await this.wallet.viewMethod({ contractId: this.contractId, method: "view_potential_winnings", args: { match_id: matchId, team: team, bet_amount: betAmount}})
    return potentialWinnings
  }

  async makeBet(match_id, decision, betAmount) {
    const deposit = utils.format.parseNearAmount(betAmount)
    return await this.wallet.callMethod({contractId: this.contractId, method: "make_bet", args: {match_id: match_id, decision: decision}, deposit})
  }
}