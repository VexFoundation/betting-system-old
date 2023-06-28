import 'regenerator-runtime/runtime';
import MatchList from './MatchList';
import logo from './Vex-pic.png';


const App = ({ isSignedIn, theContract, wallet }) => {

  const signIn = () => { wallet.signIn() }

  const signOut = () => { wallet.signOut() }

  return (
    <div className='app'>
        <div className='top'>
        <img src={logo} alt="Vex Logo" className='logo' />
          { isSignedIn 
          ? <button className='log' onClick={signOut}>Log out</button>
          : <button className ='log' onClick={signIn}>Log in</button>
        }
        </div>

        <div className='content'>
            <MatchList isSignedIn={isSignedIn} theContract={theContract}/>
        </div>
      </div>
  );
};

export default App;