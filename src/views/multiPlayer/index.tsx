import * as React from 'react';
import autobind from 'autobind-decorator';
import { connect } from 'react-redux';
import { Dispatch } from 'redux';
import { onUsers } from './api';
import { match, withRouter, RouteComponentProps } from 'react-router-dom';
import { ReduxState } from 'reducers';

interface Props extends RouteComponentProps<{}> {
  match: match<{}>;
}

interface ReduxProps extends Props {
  dispatch: Dispatch<Props>;
  socket: SocketIOClient.Socket;
  users: User[];
  gameType: string;
  gameState: GameState;
}

interface State {
  roomName: string;
  username: string;
}

@autobind
class MultiPlayer extends React.Component<ReduxProps, State> {

  constructor(props: ReduxProps) {
    super(props);

    this.state = {
      roomName: '',
      username: '',
    };
    this.props.dispatch(onUsers(this.props.socket));
  }

  setRoomName(event: React.ChangeEvent<HTMLInputElement>): void {
    this.setState({roomName: event.target.value});
  }

  setUsername(event: React.ChangeEvent<HTMLInputElement>): void {
    this.setState({username: event.target.value});
  }

  host(event: React.MouseEvent<HTMLInputElement>): void {
    event.preventDefault();
    this.props.socket.emit('joinRoom', {username: this.state.username, roomName: this.state.roomName});
    this.props.history.push(`${this.props.match.url}/${this.state.roomName}`);
  }

  render() {
    return (
      <div>
        multiPlayer
        <div>
          <form>
            <label>
              Room name
              <input value={this.state.roomName} onChange={this.setRoomName}/>
            </label>
            <br/>
            <label>
              Username
              <input value={this.state.username} onChange={this.setUsername}/>
            </label>
            <br/>
            <input type="submit" value="Enter" onClick={this.host}/>
          </form>
        </div>
      </div>
    );
  }
}

function mapStateToProps(state: ReduxState, _ownProps: Props) {
  return state.multiPlayer;
}

function mapDispatchToProps(dispatch: Dispatch<Props>) {
  return { dispatch };
}

export default connect(mapStateToProps, mapDispatchToProps)(withRouter(MultiPlayer));
