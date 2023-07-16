import { useState, useEffect } from 'react';
import BetForm from './BetForm';

const MatchList = ({ isSignedIn, theContract}) => {
  const [currentMatch, setCurrentMatch] = useState(['', '', '', '', '', '', '', '']);
  const [matchSelected, setMatchSelected] = useState(false)
  const showBetForm = isSignedIn && matchSelected
  const [matches, setMatches] = useState([]);


  useEffect(() => {
    theContract.getMatches().then(setMatches);
  }, []);

  const handleClick = (e, match, matchSelected) => {
    e.preventDefault();
    setCurrentMatch(match);
    setMatchSelected(matchSelected);
  }

  return ( 
    <div className='bet-form-content'>

      <div className="left">
          <div className="match-list">
            {matches.map(match => (
              <div key={match[0]}>
                
                {<button className="match-preview" onClick={(e) => handleClick(e, match, true)} disabled={!isSignedIn}><div>
                    <h2> {match[1]} vs {match[3]} </h2>
                    <p> {match[2]} to {match[4]} </p> 
                  </div>
                </button> }

              </div>
            ))}
          </div>
        </div>


      { showBetForm && <div className="right">
        { showBetForm && <BetForm currentMatch={currentMatch} theContract={theContract}/> }
      </div> }

      

    </div>
   );
}

 
export default MatchList;
