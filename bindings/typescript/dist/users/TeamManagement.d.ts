import React from 'react';
import { Team } from './types';
interface TeamManagementProps {
    onTeamSelect?: (team: Team) => void;
    onTeamCreate?: (team: Team) => void;
    onTeamUpdate?: (team: Team) => void;
    onTeamDelete?: (teamId: string) => void;
    className?: string;
}
export declare const TeamManagement: React.FC<TeamManagementProps>;
export default TeamManagement;
//# sourceMappingURL=TeamManagement.d.ts.map