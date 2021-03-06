/**
 * Copyright 2020 Google LLC
 * Copyright 2020 Team Spacecat
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import { getApiKey } from "./auth";
import { queryServer, startNonRating, queryNonRatingRuns } from "./utils";

const resultsElem = document.getElementById('results') as HTMLElement;
const refreshElem = document.getElementById('refresh') as HTMLButtonElement;
const missingRunElem = document.getElementById('run_missing') as HTMLButtonElement;

const MY_TEAM_ID = '3dfa39ba-93b8-4173-92ad-51da07002f1b';
const OUR_BOTS: Array<string> = [
    'tanakh_super_bot',
];
const TEAM_SIZE = 30;

function startMissingResults(): void {
    refreshElem.disabled = true;
    missingRunElem.disabled = true;
    try {
        let [subIdToTeamName, resultsAtk, resultsDef] = getResults();
        let [currentBots, subIdToBranch, subidToCommit, subidToCommitMsg, activeSub] = getOurLatestBots();
        let topPlayers = getOpponents();
        let botIDs = Object.values(currentBots);
        if (!botIDs.includes(activeSub)) {
            botIDs.unshift(activeSub);
        }

        for (var ourSubId of botIDs) {
            for (var [oppName, oppSubId] of topPlayers) {
                if (ourSubId in resultsAtk && oppSubId in resultsAtk[ourSubId]) {
                    // Already exist
                } else {
                    startNonRating(ourSubId, oppSubId);
                    // Query
                }
                if (ourSubId in resultsDef && oppSubId in resultsDef[ourSubId]) {
                    // Already exist
                } else {
                    startNonRating(oppSubId, ourSubId);
                }
            }
        }
    } finally {
        refreshElem.disabled = true;
        missingRunElem.disabled = false;
        loadResults();
    }
}

function loadResults(): void {
    try {
        let [subIdToTeamName, resultsAtk, resultsDef] = getResults();
        let [currentBots, subIdToBranch, subidToCommit, subidToCommitMsg, activeSub] = getOurLatestBots();
        let topPlayers = getOpponents();

        let botIDs = Object.values(currentBots);
        if (!botIDs.includes(activeSub)) {
            botIDs.push(activeSub);
        }

        let head = [];
        for (var sub of botIDs) {
            let name = subIdToBranch[sub];
            const label = (sub == activeSub) ? '<br>[ACTIVE]' : '';
            const commit = subidToCommit[sub];
            const commitMsg = subidToCommitMsg[sub];
            head.push(`<th><span title="${commitMsg}">${sub}<br>${commit.substring(0, 6)}<br>atk${label}</span></th>`);
            head.push(`<th><span title="${commitMsg}">${sub}<br>${commit.substring(0, 6)}<br>def${label}</span></th>`);
        }

        let rows: Array<string> = [];
        rows.push('<tr><th></th>' + head.join('') + '</tr>');
        for (var [oppName, oppSubId] of topPlayers) {
            let result = "<tr><td>" + oppName + " (" + oppSubId + ")</td>";
            for (var ourSubId of botIDs) {
                if (ourSubId in resultsAtk && oppSubId in resultsAtk[ourSubId]) {
                    let [status, playerKey] = resultsAtk[ourSubId][oppSubId];
                    let url = 'https://icfpcontest2020.github.io/#/visualize?playerkey=' + playerKey;
                    result += '<td class=' + status.toLowerCase() + '><a href="' + url + '">' + status + '</a></td>';
                } else {
                    result += "<td></td>";
                }
                if (ourSubId in resultsDef && oppSubId in resultsDef[ourSubId]) {
                    let [status, playerKey] = resultsDef[ourSubId][oppSubId];
                    let url = 'https://icfpcontest2020.github.io/#/visualize?playerkey=' + playerKey;
                    result += '<td class=' + status.toLowerCase() + '><a href="' + url + '">' + status + '</a></td>';
                } else {
                    result += "<td></td>";
                }
            }
            result += "</tr>";
            rows.push(result);
        }
        resultsElem.innerHTML = rows.join('');
    } finally {
        refreshElem.disabled = false;
        missingRunElem.disabled = false;
    }
}

function getOpponents(): Array<[string, number]> {
    const scores = <Scoreboard>JSON.parse(queryServer('/scoreboard'));
    let submissions: Array<[number, string, number]> = [];
    let oldones: Array<[string, number]> = [];
    for (var team of scores.teams) {
        if (team.team.teamId == MY_TEAM_ID) {
            continue;
        }
        const name = team.team.teamName;
        const score = team.score;
        let latestKey: number = 0;
        for (var k in team.tournaments) {
            if (parseInt(k) > latestKey) {
                latestKey = parseInt(k);
            }
        }
        const subid = team.tournaments[latestKey.toString()].submission.submissionId;
        for (var k in team.tournaments) {
            if (subid == team.tournaments[k].submission.submissionId) {
                continue;
            }
            /*
            if (team.tournaments[k].score == 50) {
                oldones.push([name + " (Top in round " + k + ")", team.tournaments[k].submission.submissionId])
            }
            if (team.tournaments[k].score == 46) {
                oldones.push([name + " (Second in round " + k + ")", team.tournaments[k].submission.submissionId])
            }
            if (team.tournaments[k].score == 42) {
                oldones.push([name + " (Third in round " + k + ")", team.tournaments[k].submission.submissionId])
            }
            */
        }
        submissions.push([score, name, subid]);
    }

    let ret: Array<[string, number]> = [];
    submissions.sort((a, b) => b[0] - a[0]);
    for (var [score, name, subid] of submissions.slice(0, TEAM_SIZE)) {
        ret.push([name, subid]);
    }
    return ret.concat(oldones);
}

function getOurLatestBots(): [Record<string, number>, Record<number, string>, Record<number, string>, Record<number, string>, number] {
    const submissions = <Array<Submission>>JSON.parse(queryServer('/submissions'));
    let currentBots: Record<string, number> = {};
    let subidToBranch: Record<number, string> = {};
    let subidToCommit: Record<number, string> = {};
    let subidToCommitMsg: Record<number, string> = {};
    let activeSub: number = 0;
    for (let i = 0; i < submissions.length; i++) {
        const sub = submissions[i];
        subidToCommit[sub.submissionId] = sub.commitHash;
        subidToCommitMsg[sub.submissionId] = sub.commitMessage;
        if (sub.active) {
            activeSub = sub.submissionId;
        }
        if (!sub.branchName) {
            continue;
        }
        if (sub.status != 'Succeeded') {
            continue;
        }
        subidToBranch[sub.submissionId] = sub.branchName;
        if (OUR_BOTS.includes(sub.branchName) && (i < 20 || sub.active)) {
            currentBots[i] = sub.submissionId;
        }
    }
    return [currentBots, subidToBranch, subidToCommit, subidToCommitMsg, activeSub];
}

function getResults(): [Record<number, string>, Record<number, Record<number, [string, number]>>, Record<number, Record<number, [string, number]>>] {
    let cachedGames: Array<Game> = [];
    const cache = localStorage.getItem('cached_games');
    if (cache) {
        cachedGames = JSON.parse(cache) as Array<Game>;
    }

    let games: Array<Game> = [];
    let prevDate = '';
    loop: while (true) {
        const ret = <GamesList>JSON.parse(queryNonRatingRuns(prevDate));
        games = games.concat(ret.games);
        if (cachedGames.length > 0) {
            const knownGame = cachedGames[0];
            for (let i = 0; i < games.length; i++) {
                if (games[i].gameId === knownGame.gameId) {
                    const restSize = games.length - i;
                    games = games.concat(cachedGames.slice(restSize));
                    break loop;
                }
            }
        }
        if (ret.hasMore && ret.next) {
            prevDate = ret.next;
            continue;
        }
        break;
    }
    localStorage.setItem('cached_games', JSON.stringify(games));

    let subidToTeamName: Record<number, string> = {};
    let resultsAtk: Record<number, Record<number, [string, number]>> = {};
    let resultsDef: Record<number, Record<number, [string, number]>> = {};
    for (var game of games) {
        const atkTeamName = game.attacker.team.teamName;
        const atkSubId = game.attacker.submissionId;
        const defTeamName = game.defender.team.teamName;
        const defSubId = game.defender.submissionId;

        if (game.attacker.team.teamId == MY_TEAM_ID && game.defender.team.teamId == MY_TEAM_ID) {
            continue;
        }

        let myTeamSubId: number = 0;
        let oppTeamName: string = '';
        let oppSubId: number = 0;
        let mySide: string = '';
        let results: Record<number, Record<number, [string, number]>> = {};
        let playerKey: number = 0;
        if (game.attacker.team.teamId == MY_TEAM_ID) {
            myTeamSubId = atkSubId;
            oppTeamName = defTeamName;
            oppSubId = defSubId;
            mySide = 'Attacker';
            results = resultsAtk;
            playerKey = game.attacker.playerKey;
        } else {
            myTeamSubId = defSubId;
            oppTeamName = atkTeamName;
            oppSubId = atkSubId;
            mySide = 'Defender';
            results = resultsDef;
            playerKey = game.defender.playerKey;
        }

        subidToTeamName[oppSubId] = oppTeamName;
        let result: string;
        if (!game.winner) {
            result = 'Pending';
        } else if (game.winner == mySide) {
            result = 'Win';
        } else if (game.winner == 'Nobody') {
            result = 'Draw';
        } else {
            result = 'Lose';
        }

        if (!(myTeamSubId in results)) {
            results[myTeamSubId] = {};
        }
        results[myTeamSubId][oppSubId] = [result, playerKey];
    }
    return [subidToTeamName, resultsAtk, resultsDef];
}

interface Submission {
    submissionId: number,
    branchName?: string,
    status: string,
    commitHash: string
    commitMessage: string
    active: boolean,
}

interface Scoreboard {
    teams: Array<TeamScore>
}

interface TeamScore {
    team: Team,
    score: number,
    tournaments: Record<string, Tournament>,
}

interface Tournament {
    submission: TournamentSubmission,
    score: number,
}

interface TournamentSubmission {
    submissionId: number,
}

interface Team {
    teamId: string
    teamName: string
}

interface Player {
    submissionId: number
    team: Team,
    playerKey: number
}

interface Game {
    gameId: string,
    attacker: Player,
    defender: Player,
    winner?: string,
}

interface GamesList {
    hasMore: boolean,
    next?: string,
    games: Array<Game>
}

function init(): void {
    refreshElem.addEventListener('click', loadResults);
    missingRunElem.addEventListener('click', startMissingResults);

    if (getApiKey() != '') {
        loadResults();
        if (location.href.indexOf('__autorun__') >= 0) {
            window.setInterval(startMissingResults, 1000 * 30);
        }
    }
}

init();
