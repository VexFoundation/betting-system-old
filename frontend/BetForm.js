import { useState, useEffect } from 'react';

const BetForm = ({currentMatch, theContract}) => {
    const [betAmount, setBetAmount] = useState('');
    const [team, setTeam] = useState('');
    const matchName = currentMatch[0]
    const [potentialWinnings, setPotentialWinnings] = useState(0.0);

    const handleSubmit = (e) => {
        e.preventDefault();
        theContract.makeBet(matchName, team, betAmount)
      }

      useEffect(() => {
        setTeam('default');
        setBetAmount('')
      }, [currentMatch]);

    //   && betAmount != ''  

      useEffect(() => {
        if (betAmount == '') {
            setPotentialWinnings(0)
        }
        if (team != 'default' && team != '' && betAmount != '' ) {
            console.log(team)
            theContract.getPotentialWinnings({ matchId: matchName, team: team, betAmount: betAmount }).then(setPotentialWinnings)
        }
      }, [team, betAmount]);

    return ( 
        <div className='box'>
            <form onSubmit={handleSubmit}>
                <label className='bet-text'>Ⓝ Bet Amount</label>
                <input className='bet-input'
                type='text'
                required 
                value = { betAmount }
                onChange={(e) => setBetAmount(e.target.value)}/>

                <label className='bet-text'>Team</label>
                <select 
                    className='bet-select'
                    value={ team }
                    onChange={(e) => setTeam(e.target.value)}
                >
                    <option value={'default'}>Select Team</option>
                    <option value={ currentMatch[1] }>{ currentMatch[1] }</option>
                    <option value={ currentMatch[3] }>{ currentMatch[3] }</option>
                </select>

                <p className='pot-win'>Ⓝ Potential Winnings <br />{ potentialWinnings }</p>

                <button className='bet-button' disabled={(team == 'default')}>Make bet</button>

            </form> 
        </div>
     );
}
 
export default BetForm;