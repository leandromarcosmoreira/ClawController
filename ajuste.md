url=https://api.movidesk.com/public/v1/tickets/past?token=$api_token&$select=$select_fields&$expand=owner,createdBy,parentTickets,childrenTickets,assets,satisfactionSurveyResponses&$orderby=id%20desc&$top=$top_val&$skip=$skip_val&$filter=$filter_val

query=SELECT (SELECT value FROM settings WHERE `key` = 'API_TOKEN') AS api_token, (SELECT value FROM settings WHERE `key` = 'API_TICKET_SELECT_BASIC') AS select_fields, (SELECT value FROM settings WHERE `key` = 'API_PAGE_SIZE') AS top_val, COALESCE(NULLIF(:VAR_PAGINACAO, ''), '0') AS skip_val, CONCAT('createdDate%20lt%20', DATE_FORMAT(STR_TO_DATE(COALESCE((SELECT value FROM settings WHERE `key` = 'API_CUTOFF_DATE'), DATE_FORMAT(DATE_SUB(NOW(), INTERVAL 90 DAY), '%Y-%m-%dT%H:%i:%s.000Z')), '%Y-%m-%dT%H:%i:%s.000Z'), '%Y-%m-%dT%H:%i:%s.000Z')) AS filter_val FROM DUAL

---------------------
url=https://api.movidesk.com/public/v1/tickets?token=$api_token&$select=$select_fields&$expand=owner,createdBy,parentTickets,childrenTickets,assets,satisfactionSurveyResponses&$orderby=lastUpdate%20desc&$top=$top_val&$skip=$skip_val&$filter=$filter_val

query=SELECT (SELECT value FROM settings WHERE `key` = 'API_TOKEN') AS api_token, (SELECT value FROM settings WHERE `key` = 'API_TICKET_SELECT_BASIC') AS select_fields, (SELECT value FROM settings WHERE `key` = 'API_PAGE_SIZE') AS top_val, COALESCE(NULLIF(:VAR_PAGINACAO, ''), '0') AS skip_val, CONCAT('lastUpdate%20ge%20', DATE_FORMAT(STR_TO_DATE((SELECT value FROM settings WHERE `key` = 'API_LAST_SYNC'), '%Y-%m-%dT%H:%i:%s.000Z'), '%Y-%m-%dT%H:%i:%s.000Z')) AS filter_val FROM DUAL

---------------------
url=https://api.movidesk.com/public/v1/tickets?token=$api_token&$select=id,protocol,subject,type,category,urgency,status,baseStatus,justification,origin,originEmailAccount,createdDate,resolvedIn,reopenedIn,closedIn,lastActionDate,actionCount,lastUpdate,lifeTimeWorkingTime,stoppedTimeWorkingTime,resolvedInFirstCall,chatWidget,chatGroup,chatTalkTime,chatWaitingTime,sequence,slaAgreement,slaAgreementRule,slaSolutionTime,slaResponseTime,slaSolutionDate,slaResponseDate,slaRealResponseDate,slaSolutionChangedByUser,slaSolutionDateIsPaused,jiraIssueKey,redmineIssueId,isDeleted&$expand=owner,createdBy,clients,parentTickets,childrenTickets,ownerHistories,statusHistories,actions($expand=attachments),assets,customFieldValues,satisfactionSurveyResponses&$filter=$filter_val

query=SELECT (SELECT value FROM settings WHERE `key` = 'API_TOKEN') AS api_token, CONCAT('id%20eq%20', REPLACE(:VAR_BATCH_IDS, ',', '%20or%20id%20eq%20')) AS filter_val FROM DUAL

---------------------
url=https://api.movidesk.com/public/v1/tickets?token=$api_token&$select=id,actions&$expand=actions($expand=timeAppointments)&$filter=$filter_val

query=SELECT (SELECT value FROM settings WHERE `key` = 'API_TOKEN') AS api_token, CONCAT('id%20eq%20', REPLACE(:VAR_BATCH_IDS, ',', '%20or%20id%20eq%20')) AS filter_val FROM DUAL

---------------------
url=https://api.movidesk.com/public/v1/tickets?token=$api_token&$select=id,actions&$expand=actions($expand=expenses)&$filter=$filter_val

query=SELECT (SELECT value FROM settings WHERE `key` = 'API_TOKEN') AS api_token, CONCAT('id%20eq%20', REPLACE(:VAR_BATCH_IDS, ',', '%20or%20id%20eq%20')) AS filter_val FROM DUAL

---------------------
