# Appendix A: Complete Message Catalog

This appendix contains all RMI messages discovered through Ghidra string analysis of Rag2.exe.

**Total Messages:** 200+  
**Source:** Ghidra string extraction  
**Status:** Message IDs (numeric values) to be determined through packet capture

---

## Message Categories

1. [Authentication & Login](#1-authentication--login)
2. [Channel & Server](#2-channel--server)
3. [Character Management](#3-character-management)
4. [Combat & Skills](#4-combat--skills)
5. [Party & Social](#5-party--social)
6. [Guild](#6-guild)
7. [Items & Inventory](#7-items--inventory)
8. [Economy & Trading](#8-economy--trading)
9. [Dungeons & Instances](#9-dungeons--instances)
10. [PvP & Arena](#10-pvp--arena)
11. [Game Events](#11-game-events)
12. [Miscellaneous](#12-miscellaneous)

---

## 1. Authentication & Login

### Request Messages (Req)
- `ReqLogin` - Authenticate with credentials
- `ReqLoginChannel` - Lobby server authentication
- `ReqGraLogin` - Alternative login method
- `ReqSteamLogin` - Steam platform login
- `ReqMessengerLoginKey` - Messenger service login

### Answer Messages (Ans)
- `AnsLogin` - Login response
- `AnsLoginChannel` - Lobby authentication response
- `AnsPlayerLoginKey` - Session key distribution
- `AnsLoginMessenger` - Messenger login response

### Notify Messages (Nfy)
- `NfyServerTime` - Server time synchronization
- `NfyServerTimeToLoginPC` - Login PC time notification

### Acknowledgment Messages (Ack)
- `AckLogin` - Login acknowledged
- `AckLoginChannel` - Lobby login acknowledged
- `AckServerStatus` - Server status response
- `AckVersionCheck` - Client version validated
- `AckChannelList` - Channel list response
- `AckChannelListInGame` - In-game channel list

---

## 2. Channel & Server

### Request Messages (Req)
- `ReqServerStatus` - Query server status
- `ReqChannelList` - Get available channels
- `ReqChannelMove` - Switch channels
- `ReqChannelMoveLogin` - Channel move with login
- `ReqWorldList` - Query world list
- `ReqPing` - Ping server

### Answer Messages (Ans)
- `AnsChannelMove` - Channel move response
- `AnsChannelMoveLogin` - Channel move login response
- `AnsWorldList` - World list response

### Notify Messages (Nfy)
- `NfyChannelDisconnect` - Channel disconnection notice
- `NfyServerTimeToLoginPC` - Server time to login PC

---

## 3. Character Management

### Request Messages (Req)
- `ReqCharNameList` - Query character names
- `ReqRenameChaName` - Rename character
- `ReqCanRenameChaName` - Check if rename allowed
- `ReqCanRenameChaNameNow` - Check immediate rename
- `ReqReserveDeletePC` - Schedule character deletion
- `ReqCancelReservedDeletePC` - Cancel deletion
- `ReqReservedPCList` - List pending deletions
- `ReqReservedPCCount` - Count pending deletions
- `ReqUpdateActivePC` - Update active character
- `ReqAppearanceChange` - Change appearance
- `ReqCharTransform` - Character transformation

### Answer Messages (Ans)
- `AnsCharNameList` - Character name list
- `AnsRenameChaName` - Rename response
- `AnsCanRenameChaName` - Rename check response
- `AnsCanRenameChaNameNow` - Immediate rename check
- `AnsReserveDeletePC` - Deletion scheduled
- `AnsCancelReservedDeletePC` - Deletion cancelled
- `AnsReservedPCList` - Pending deletion list
- `AnsReservedPCCount` - Deletion count
- `AnsAppearanceChange` - Appearance change result
- `AnsPCPageCount` - Character page count

### Notify Messages (Nfy)
- `NfyCharTransform` - Character transformation notice
- `NfyPCItemTrait` - Character item trait update
- `NfyChangeProJob` - Job change notification
- `NfyChangeProJobQuest` - Job quest change
- `NfyChageProJobGuardianSkill` - Guardian skill change
- `NfyChangeProJobDelQuickSlot` - Quick slot deletion

---

## 4. Combat & Skills

### Request Messages (Req)
- `ReqSkillStart` - Start skill cast
- `ReqSkillStart_Item` - Start skill from item
- `ReqSkillDamage` - Apply skill damage
- `ReqSkillDamageTick` - Periodic damage tick
- `ReqSkillCastingStart` - Begin casting
- `ReqSkillCastingCancel` - Cancel casting
- `ReqSkillChannelingComplete` - Complete channeling
- `ReqSkillChannelingCancel` - Cancel channeling
- `ReqSkillMoveStart` - Start skill movement
- `ReqSkillMoveEnd` - End skill movement
- `ReqSkillMoveCancel` - Cancel skill movement
- `ReqSkillMoveNotify` - Notify skill movement
- `ReqFallDamage` - Apply fall damage
- `ReqUpdateDeadPoint` - Update death point

### Answer Messages (Ans)
- `AnsUseSkillStart` - Skill use started
- `AnsSkillError` - Skill error
- `AnsSkillDamageTick` - Damage tick response
- `AnsSkillCastingStart` - Casting started
- `AnsSkillCastingCancel` - Casting cancelled
- `AnsSkillMoveStart` - Movement started
- `AnsSkillMoveEnd` - Movement ended
- `AnsSkillMoveCancel` - Movement cancelled
- `AnsErrorNormalAttack` - Normal attack error

### Notify Messages (Nfy)
- `NfySkillDamage` - Skill damage notification
- `NfySkillDamageTick` - Damage tick notification
- `NfySkillDamageError` - Skill damage error
- `NfySkillStartError` - Skill start error
- `NfyUseSkillStart` - Skill use notification
- `NfySkillCastingStart` - Casting notification
- `NfySkillCastingCancel` - Cast cancel notification
- `NfySkillMoveStart` - Movement notification
- `NfySkillMoveEnd` - Movement end notification
- `NfySkillMoveCancel` - Movement cancel notification
- `NfySkillMoveNotify` - Movement update
- `NfySkillCoolTime` - Cooldown notification
- `NfyCoolTimeClear` - Cooldown cleared
- `NfyCoolTimeLockReleased` - Cooldown lock released
- `NfyAttackDamage` - Attack damage notification
- `NfyNormalAttackStart` - Normal attack started
- `NfyRevivalSkill` - Revival skill notification
- `NfyFallDamage` - Fall damage notification

---

## 5. Party & Social

### Request Messages (Req)
- `ReqPartyMemberPCOID` - Get party member ID
- `ReqPartySubMaster` - Set sub-master
- `ReqPartyDismissSubMaster` - Dismiss sub-master
- `ReqPartyFrameMove` - Move party frame
- `ReqPartyTypeChange` - Change party type
- `ReqPartyKickVote` - Vote to kick member
- `ReqPartyMemberKickVoteStart` - Start kick vote
- `ReqPartyFightingTrimVote` - Fighting trim vote
- `ReqPartyTacticsTarget` - Set tactics target
- `ReqPartyDungeonTeleport` - Dungeon teleport
- `ReqPartyDiceChattingCmd` - Dice roll command
- `ReqPartyMatching_ReJoinDungeon` - Rejoin dungeon
- `ReqPartyMatching_OutDungeon` - Leave dungeon
- `ReqPartyMatching_Into_Dungeon_Participation` - Join dungeon
- `ReqPartyMatchingSearchInvite` - Search for invite
- `ReqPartyMatchingFindDungeonJoinSelectBtn` - Join select button
- `ReqPartyMatchingFindDungeonCancel` - Cancel dungeon find
- `ReqPartyMatchingInProgressInfo` - Get progress info
- `ReqPartyMatchingFindDungeonRandom` - Find random dungeon
- `ReqPartyMatchingFindDungeon` - Find specific dungeon
- `ReqPartyMatchingInviteMsg` - Party invite message
- `ReqFindParty_Party_SelectRole` - Select party role
- `ReqFindParty_Party_SelectRole_Cancel` - Cancel role select

### Answer Messages (Ans)
- `AnsPartyMemberPCOID` - Party member ID response
- `AnsPartySubMaster` - Sub-master response
- `AnsPartyDismissSubMaster` - Dismiss response
- `AnsPartyFrameMove` - Frame move response
- `AnsPartyTypeChange` - Type change response
- `AnsPartyKickVote` - Kick vote response
- `AnsPartyMemberKickVoteStart` - Kick vote started
- `AnsPartyFightingTrim` - Fighting trim response
- `AnsPartyFightingTrimVote` - Trim vote response
- `AnsPartyTacticsTarget` - Tactics target response
- `AnsPartyTacticsTargetError` - Tactics error
- `AnsPartyMatching_ReJoinDungeon` - Rejoin response
- `AnsPartyMatching_OutDungeon` - Out dungeon response
- `AnsPartyMatchingFindDungeonJoinSelectBtn` - Join select response
- `AnsPartyMatchingFindDungeonCancel` - Cancel response
- `AnsPartyMatchingInProgressInfo` - Progress info
- `AnsPartyMatchingFindDungeonRandom` - Random dungeon response
- `AnsPartyMatchingFindDungeon` - Dungeon find response
- `AnsFindParty_Party_SelectRole` - Role select response
- `AnsFindParty_Party_SelectRole_Cancel` - Role cancel response

### Notify Messages (Nfy)
- `NfyPartySubMaster` - Sub-master notification
- `NfyPartyDismissSubmaster` - Dismiss notification
- `NfyPartyFrameMove` - Frame move notification
- `NfyPartyTypeChange` - Type change notification
- `NfyPartyMemberKickVote` - Kick vote notification
- `NfyResultPartyKickVote` - Kick vote result
- `NfyPartyFightingTrimStart` - Fighting trim start
- `NfyPartyFightingTrimResult` - Fighting trim result
- `NfyPartyItemListClose` - Item list closed
- `NfyPartyDiceChattingCmd` - Dice roll notification
- `NfyPartyJurisdictionIndun` - Dungeon jurisdiction
- `NfyDungeonFind_Party_Cancel` - Dungeon find cancelled
- `NfyDungeonFind_InProgress_JoinMsg` - Join in progress
- `NfyDungeonFind_PC_Private_Indun_Join` - Private dungeon join
- `NfyFindParty_Party_Dungeon_SelectRolePlease` - Please select role
- `NfyPartyMatchingFindDungeonJoin_Fail` - Join failed
- `NfyPartyMatchingFindDungeonStartJoinMessage` - Join started
- `NfyResult_FindParty_Party_SelectRole` - Role select result

---

## 6. Guild

### Request Messages (Req)
- `ReqGuildStorage` - Access guild storage
- `ReqGuildSkillList` - Get guild skill list
- `ReqGuildSkillLearn` - Learn guild skill
- `ReqGuildSkillLearnBook` - Learn from skill book
- `ReqGuildSetBribePercent` - Set bribe percentage
- `ReqGuildTaxInquiry` - Query guild tax
- `ReqGuildTaxWithdraw` - Withdraw guild tax
- `ReqGuildListRegisteredRobe` - List registered robes
- `ReqGuildSelectRobe` - Select guild robe
- `ReqGuildEmergencyCallAnswer` - Answer emergency call
- `ReqRenameGuildName` - Rename guild
- `ReqCanRenameGuildName` - Check rename allowed
- `ReqWithdrawUnion` - Withdraw from union
- `ReqJoinUnion` - Join union
- `ReqGuildHouseTeleport` - Guild house teleport
- `ReqGuildHouseLotteryRank` - Lottery rank
- `ReqGuildHouseResult` - House result
- `ReqGuildHouseTicket` - Get house ticket
- `ReqGuildHouseLottery` - Guild house lottery

### Answer Messages (Ans)
- `AnsGuildStorage` - Guild storage response
- `AnsGuildSkillList` - Guild skill list
- `AnsGuildSkillLearn` - Skill learn response
- `AnsGuildSkillLearnBook` - Skill book response
- `AnsGuildSetBribePercent` - Bribe set response
- `AnsGuildTaxInquiry` - Tax inquiry response
- `AnsGuildTaxWithdraw` - Tax withdraw response
- `AnsGuildListRegisteredRobe` - Robe list
- `AnsGuildSelectRobe` - Robe select response
- `AnsGuildEmergencyCallAsk` - Emergency call ask
- `AnsRenameGuildName` - Rename response
- `AnsCanRenameGuildName` - Rename check response
- `AnsWithdrawUnion` - Union withdraw response
- `AnsJoinUnion` - Union join response
- `AnsGuildHouseTeleport` - House teleport response
- `AnsGuildHouseLotteryRank` - Lottery rank
- `AnsGuildHouseResult` - House result
- `AnsGuildHouseTicket` - House ticket response
- `AnsGuildHouseLottery` - Lottery response
- `AnsGuildDelPossible` - Deletion possible check
- `AnsGuildGMInitSkillList` - GM skill list init
- `AnsOpenGuildStorage` - Open storage response
- `AnsGuildStorageItemPut` - Put item response
- `AnsGuildStorageItemGet` - Get item response
- `AnsGuildFundsInHandDeposit` - Deposit funds
- `AnsGuildFundsInHandWithdraw` - Withdraw funds
- `AnsGuildStorageStatement_Item` - Item statement
- `AnsGuildStorageStatement_Zeny` - Zeny statement

### Notify Messages (Nfy)
- `NfyGuildSkillLearn` - Skill learned notification
- `NfyGuildBribeHunting` - Bribe hunting notice
- `NfyGuildBribePCLevelUp` - Bribe level up
- `NfyGuildLevelUP` - Guild level up
- `NfyGuildHandoverMemberBribePoint_ToGuildMaster` - Bribe point handover
- `NfyGuildMasterHandover` - Master handover
- `NfyGuildMemberModifyAuthority` - Member authority change
- `NfyGuildModifyAuthority` - Authority modification
- `NfyGuildDelAuthority` - Authority deletion
- `NfyGuildRobeChanged` - Robe changed
- `NfyGuildHouseAnnounce` - House announcement
- `NfyUpdateGuildEmblem` - Emblem updated
- `NfyUpdateGuildFundsInHand` - Funds updated
- `NfyAddGuildStorage_Item` - Item added to storage
- `NfyDelGuildStorage_Item` - Item removed from storage
- `NfyGuildInviteYouReq` - Guild invite request

---

## 7. Items & Inventory

### Request Messages (Req)
- `ReqUseCashItem` - Use cash shop item
- `ReqUseDyeItem` - Use dye item
- `ReqItemTransform` - Transform item
- `ReqGiveEffectToItem` - Give effect to item
- `ReqExtendTimeSetItem` - Extend time-limited item
- `ReqRidingExtendItem` - Extend riding item
- `ReqSealItem` - Seal item
- `ReqSealItemClear` - Clear item seal
- `ReqExtractSocket` - Extract socket
- `ReqAttachSocket` - Attach socket
- `ReqOpenItemSocket` - Open item socket
- `ReqRefineItem` - Refine item
- `ReqRefineProbability` - Get refine probability
- `ReqRandomRefineItem` - Random refine
- `ReqRandomRefineCommission` - Refine commission
- `ReqChangeRandomRefineShopName` - Change refine shop name
- `ReqRefineShopOpen` - Open refine shop
- `ReqOpenRefineShopBuyerUI` - Open buyer UI
- `ReqChangeRefineShopName` - Change shop name
- `ReqBeginRefineShop` - Begin refine shop
- `ReqTransferReinforceItem` - Transfer reinforcement
- `ReqDirectReInforce` - Direct reinforcement
- `ReqRandomEnforceItem` - Random enforcement
- `ReqRandomEnforceItemConfirm` - Confirm random enforcement
- `ReqCardEnforceItem` - Card enforcement
- `ReqCardEnforceProbability` - Card enforce probability
- `ReqCardBookPut` - Put card in book
- `ReqCardBookTakeOff` - Take off card
- `ReqCardBookTakeOff_1Pcs` - Take off 1 piece
- `ReqBestrowAttribute` - Bestow attribute
- `ReqOutfitChangeItem` - Change outfit item
- `ReqOutfitDivideReleaseItem` - Release outfit divide
- `ReqGiveItemRemainTime` - Get item remain time
- `ReqGrindingItemReset` - Reset grinding item
- `ReqInitGrindingOpt` - Init grinding option

### Answer Messages (Ans)
- `AnsUseCashItem` - Cash item use response
- `AnsUseDyeItem` - Dye item use response
- `AnsItemTransform` - Transform response
- `AnsGiveEffectToItem` - Effect given response
- `AnsExtendTimeSetItem` - Time extend response
- `AnsRidingExtendItem` - Riding extend response
- `AnsSealItem` - Seal item response
- `AnsSealItemClear` - Clear seal response
- `AnsExtractSocket` - Extract socket response
- `AnsAttachSocket` - Attach socket response
- `AnsOpenItemSocket` - Open socket response
- `AnsRefineItem` - Refine response (not in dump)
- `AnsRefineProbability` - Probability response (not in dump)
- `AnsRandomRefineItem` - Random refine response (not in dump)
- `AnsRandomRefineCommission` - Commission response (not in dump)
- `AnsChangeRandomRefineShopName` - Shop name change response
- `AnsRefineShopOpen` - Shop open response (not in dump)
- `AnsOpenRefineShopBuyerUI` - Buyer UI response (not in dump)
- `AnsChangeRefineShopName` - Shop name change response (not in dump)
- `AnsBeginRefineShop` - Begin shop response
- `AnsTransferReinforceItem` - Transfer response
- `AnsDirectReInforce` - Direct reinforce response
- `AnsRandomEnforceItem` - Random enforce response
- `AnsRandomEnforceItemConfirm` - Confirm response
- `AnsCardBookPut` - Card put response
- `AnsCardBookTakeOff` - Card takeoff response
- `AnsCardBookTakeOff_1Pcs` - 1 piece takeoff response
- `AnsBestrowAttribute` - Bestow attribute response
- `AnsOutfitChangeItem` - Outfit change response
- `AnsOutfitDivideReleaseItem` - Outfit divide release response
- `AnsGiveItemRemainTime` - Remain time response
- `AnsGrindingItemReset` - Grinding reset response
- `AnsInitGrindingOpt` - Init grinding response

### Notify Messages (Nfy)
- `NfyItemUpdate` - Item update notification
- `NfyTimedItemInfo` - Timed item info
- `NfyExpiredItemInfo` - Expired item info
- `NfyTimeSetItemStart` - Time-limited item start
- `NfyTimeSetItemEnd` - Time-limited item end
- `NfyCardEquipTakeOff` - Card equip takeoff
- `NfyCardDurabilityUpdate` - Card durability update
- `NfyCardBookChargedTime` - Card book charged time
- `NfyCardBookChargedTimeEnd` - Charged time end
- `NfyCardBookAlramEndTime` - Alarm end time
- `NfyRefineItemNotice` - Refine item notice
- `NfyRefineCommissionToBuyer` - Refine commission notice
- `NfyChangeRefineShopName` - Refine shop name change
- `NfyBeginRefineShop` - Begin refine shop notice
- `NfyChangeRandomRefineShopName` - Random refine shop name change
- `NfyRandomRefineCommissionToBuyer` - Random refine commission
- `NfyPCEquipView` - Equipment view update

---

## 8. Economy & Trading

### Request Messages (Req)
- `ReqAddPrivateShopItem` - Add item to private shop
- `ReqDeletePrivateShopItem` - Delete shop item
- `ReqAddPrivateShopCraft` - Add craft to shop
- `ReqDeletePrivateShopCraft` - Delete shop craft
- `ReqOpenPrivateShopSellerUI` - Open seller UI
- `ReqBeginPrivateShop` - Begin private shop
- `ReqChangePrivateShopName` - Change shop name
- `ReqEndPrivateShop` - End private shop
- `ReqOpenPrivateShopBuyerUI_ItemList` - Open buyer item list
- `ReqOpenPrivateShopBuyerUI_CraftList` - Open buyer craft list
- `ReqBuyPrivateShopItem` - Buy shop item
- `ReqBuyPrivateShopCraft` - Buy shop craft
- `ReqClosePrivateShopBuyerUI` - Close buyer UI
- `ReqRandomItemBatting` - Random item batting
- `ReqGameMallUpdate` - Update game mall
- `ReqGameMallMyInfo` - Get mall info
- `ReqGameMallItemSalesCount` - Get item sales count
- `ReqGameMallPurchaseItemList` - Get purchase list
- `ReqGameMallItemList` - Get item list
- `ReqGameMallItemListAll` - Get all items
- `ReqGameMallItemPrice` - Get item price
- `ReqGameMallSearchItem` - Search mall item
- `ReqGameMallFindChar` - Find character in mall
- `ReqGameMallPurchase` - Purchase from mall

### Answer Messages (Ans)
- `AnsAddPrivateShopItem` - Add item response
- `AnsDeletePrivateShopItem` - Delete item response
- `AnsAddPrivateShopCraft` - Add craft response
- `AnsDeletePrivateShopCraft` - Delete craft response
- `AnsOpenPrivateShopSellerUI` - Open seller UI response
- `AnsBeginPrivateShop` - Begin shop response
- `AnsChangePrivateShopName` - Change name response
- `AnsEndPrivateShop` - End shop response
- `AnsOpenPrivateShopBuyerUI_ItemList` - Buyer item list
- `AnsOpenPrivateShopBuyerUI_CraftList` - Buyer craft list
- `AnsBuyPrivateShopItem` - Buy item response
- `AnsBuyPrivateShopCraft` - Buy craft response
- `AnsClosePrivateShopBuyerUI` - Close buyer UI response
- `AnsRandomItemSelected` - Random item selected
- `AnsRandomItemInBoxList` - Random box item list
- `AnsGameMallUpdate` - Mall update response
- `AnsGameMallMyInfo` - Mall info response
- `AnsGameMallItemSalesCount` - Sales count response
- `AnsGameMallPurchaseItemList` - Purchase list response
- `AnsGameMallItemList` - Item list response
- `AnsGameMallItemListAll` - All items response
- `AnsGameMallItemPrice` - Item price response
- `AnsGameMallSearchItem` - Search response
- `AnsGameMallFindChar` - Find character response
- `AnsGameMallPurchase` - Purchase response
- `AnsActorIconInfo` - Actor icon info
- `AnsWebItemError` - Web item error
- `AnsWebItemAtMyInven` - Web item in inventory

### Notify Messages (Nfy)
- `NfyBeginPrivateShop` - Begin shop notification
- `NfyEndPrivateShop` - End shop notification
- `NfyChangePrivateShopName` - Name change notification
- `NfyTerminatePrivateShop` - Shop terminated
- `NfyTerminatePrivateShopForBuyer` - Terminated for buyer
- `NfyAnsBuyPrivateShopItem` - Buy item notification
- `NfyPrivateShopItemSaled` - Item sold notification
- `NfyPrivateShopSaledList` - Sold list notification
- `NfyCraftSaleShopItemBuy` - Craft sale buy
- `NfyAuctionSystemMessage` - Auction system message
- `NfyWebMallItems` - Web mall items update
- `NfyCurrencyChanged` - Currency changed

---

## 9. Dungeons & Instances

### Request Messages (Req)
- `ReqDungeonReset` - Reset dungeon
- `ReqDungeonLevelSelect` - Select dungeon level
- `ReqDungeonEnteranceInfo` - Get entrance info
- `ReqNextStageLevel` - Move to next stage
- `ReqChallengeNextStage` - Challenge next stage
- `ReqQuestionDungeonParticipationJoin` - Question dungeon join
- `ReqQuestionAnswer_DungeonParticipationJoin` - Answer question
- `ReqIllusionPyramidGetMyState` - Get pyramid state
- `ReqIllusionPyramidDrawStage` - Draw pyramid stage
- `ReqIllusionPyramidGetResult` - Get pyramid result
- `ReqIllusionPyramidFinish` - Finish pyramid
- `ReqIllusionPyramidItemInfo` - Get pyramid item info
- `ReqTRoomJoin` - Join training room
- `ReqTRoomIn` - Enter training room
- `ReqTRoomOut` - Exit training room
- `ReqTRoomInCancel` - Cancel training room entry

### Answer Messages (Ans)
- `AnsDungeonReset` - Reset response
- `AnsDungeonLevelSelect` - Level select response (not in dump)
- `AnsDungeonEnteranceInfo` - Entrance info response
- `AnsIllusionPyramidGetMyState` - Pyramid state response
- `AnsIllusionPyramidDrawStage` - Draw stage response
- `AnsIllusionPyramidGetResult` - Pyramid result response
- `AnsIllusionPyramidFinish` - Finish response
- `AnsIllusionPyramidItemInfo` - Pyramid item info
- `AnsTRoomIn` - Training room in response
- `AnsTRoomInCancel` - Training room cancel response

### Notify Messages (Nfy)
- `NfyDungeonBindReset` - Dungeon bind reset
- `NfyDungeonBindResetWarning` - Reset warning
- `NfyDungeonBindMissionInfo` - Mission info
- `NfyDungeonLevelSelectInfo` - Level select info
- `NfyJoinDungeonIsRandom` - Random dungeon join
- `NfyRandomDungeonCompleteGift` - Random dungeon reward
- `NfyEndOfStageDungeon` - End of stage
- `NfyBeginLevelOnStage` - Begin level on stage
- `NfyFailureLevelOnStage` - Failure on stage
- `NfyReadyOkNextLevelOnStage` - Ready for next level
- `NfyTRoomState` - Training room state
- `NfyTRoomJoinMsg` - Training room join message
- `NfyTRoomPartyState` - Training room party state
- `NfyTRoomBattleState` - Training room battle state
- `NfyTRoomBattleStateAll` - All battle state
- `NfyTRoomPcKill` - Training room PC kill
- `NfyTRoomNpcKill` - Training room NPC kill
- `NfyTRoomEnd` - Training room end
- `NfyTRoomNpcKillByNpc` - NPC killed by NPC
- `NfyTRoomPcKillByNpc` - PC killed by NPC
- `NfyTRoomEffectList` - Training room effect list
- `NfyTRoomRematchingParty` - Rematching party

---

## 10. PvP & Arena

### Request Messages (Req)
- `ReqFieldPvP` - Request field PvP
- `ReqAcceptFiledPvP` - Accept field PvP
- `ReqCancelFiledPvP` - Cancel field PvP
- `ReqCountOverFieldPvP` - Count over field PvP
- `ReqColosseumJoin` - Join colosseum
- `ReqColosseumInCancel` - Cancel colosseum (not in Req dump)

### Answer Messages (Ans)
- `AnsReqErrorFiledPvP` - Field PvP error
- `AnsCertifyFieldPvP` - Certify field PvP
- `AnsColosseumInCancel` - Colosseum cancel response

### Notify Messages (Nfy)
- `NfyStartFiledPvP` - Start field PvP
- `NfyResultFiledPvP` - Field PvP result
- `NfyAcceptFiledPvP` - Accept field PvP notification
- `NfyCancelFiledPvP` - Cancel field PvP notification
- `NfyGetOutFiledPvP` - Get out of field PvP
- `NfyGetInFiledPvP` - Get into field PvP
- `NfyColosseumStart` - Colosseum start
- `NfyColosseumBeforehandStart` - Colosseum beforehand start
- `NfyColosseumWin` - Colosseum win
- `NfyColosseumJoinCount` - Colosseum join count
- `NfyColosseumRoundStateChange` - Round state change
- `NfyColosseumRoundClear` - Round clear
- `NfyColosseumLimitJoin` - Limit join
- `NfyColosseumRoundStartPlayerList` - Round start player list
- `NfyColosseumRankingUpdate` - Ranking update
- `NfyColosseumPCKillCount` - PC kill count
- `NfyColosseumScoreBoard` - Score board
- `NfyColosseumJoinMsg` - Join message
- `NfyColosseumPartyState` - Party state
- `NfyColosseumRematchingParty` - Rematching party
- `NfyEntryPartyError` - Entry party error
- `NfyPvpPCInfo` - PvP PC info

---

## 11. Game Events

### Request Messages (Req)
- `ReqMyEvent` - Get my events
- `ReqAttendanceReward` - Get attendance reward
- `ReqAttendanceRewardGetItem` - Get attendance item
- `ReqShowAttendanceState` - Show attendance state
- `ReqPlayTimeReward` - Get play time reward
- `ReqPlayTimeRewardGetItem` - Get play time item
- `ReqMakeOfferings` - Make offerings
- `ReqGetChanceRandomboxPrizeInfo` - Get chance box prize
- `ReqGetSuperChanceRandomboxState` - Get super chance state
- `ReqGetFeverTimeRandomboxState` - Get fever time state
- `ReqJumpingLevelup` - Jumping level up
- `ReqReviewPC` - Review player character
- `ReqAllowAnyoneReviewMe` - Allow anyone to review

### Answer Messages (Ans)
- `AnsMyEvent` - My events response
- `AnsAttendanceReward` - Attendance reward response
- `AnsAttendanceRewardGetItem` - Attendance item response
- `AnsShowAttendanceState` - Attendance state response
- `AnsPlayTimeReward` - Play time reward response
- `AnsPlayTimeRewardGetItem` - Play time item response
- `AnsMakeOfferings` - Offerings response
- `AnsGetChanceRandomboxPrizeInfo` - Chance box prize info
- `AnsGetSuperChanceRandomboxState` - Super chance state
- `AnsGetFeverTimeRandomboxState` - Fever time state
- `AnsJumpingLevelup` - Jumping level up response
- `AnsReviewPC` - Review PC response
- `AnsAllowAnyoneReviewMe` - Allow review response

### Notify Messages (Nfy)
- `NfyShowAttendanceState` - Show attendance state
- `NfyEventEnter` - Event enter notification
- `NfyEventClear` - Event clear notification
- `NfyWebEventWinnerNotice` - Web event winner notice
- `NfyCountLimitItemNotice` - Count limit item notice
- `NfyEventSpawnNDropNotice` - Event spawn and drop notice
- `NfyWorldEventTrigger` - World event trigger
- `NfyMarketRiseUpEnd` - Market rise up end

---

## 12. Miscellaneous

### Request Messages (Req)
- `ReqCheatTool` - Cheat tool (debug/GM)
- `ReqAbilValue` - Get ability value
- `ReqInitSimulationStat` - Init simulation stat
- `ReqLoadQuestTraceList` - Load quest trace list
- `ReqSaveQuestTraceList` - Save quest trace list
- `ReqTeleportSavePoint` - Teleport to save point
- `ReqMultiTeleportNpc` - Multi-teleport NPC
- `ReqGliderTeleport` - Glider teleport
- `ReqTakeFarGlider` - Take far glider
- `ReqRideRidingPet` - Ride pet
- `ReqRanking` - Get ranking
- `ReqRankingList` - Get ranking list
- `ReqRankerList` - Get ranker list
- `ReqRankerBuff` - Get ranker buff
- `ReqHelperDeclare` - Helper declare
- `ReqCutSceneStart` - Start cutscene
- `ReqCutSceneEnd` - End cutscene
- `ReqJobMasteryStart` - Start job mastery
- `ReqJobMasteryTraining` - Job mastery training
- `ReqJobMasteryUpgrade` - Upgrade job mastery
- `ReqRegRevivalMercenary` - Register revival mercenary
- `ReqUpdatePCPos` - Update PC position (not in Req dump)
- `ReqPCollection` - Pet collection
- `ReqPCollectionSelected` - Pet collection selected
- `ReqPCollectionCombinEggBlank` - Combine egg blank
- `ReqPCollectionUsePetEgg` - Use pet egg
- `ReqPCollectionActivePet` - Active pet
- `ReqPCollectionSetActivePet` - Set active pet
- `ReqPCollectionIncubator` - Pet incubator
- `ReqPCollectionEggIntoIncubator` - Egg into incubator
- `ReqPCollectionCollectEgg` - Collect egg
- `ReqMCollection` - Monster collection
- `ReqMCollectionSelected` - Monster collection selected
- `ReqMCollectionCombinProtein` - Combine protein
- `ReqMCollectionUseProtein` - Use protein
- `ReqThemeCostumeCompose` - Theme costume compose
- `ReqSiegeArmsUseStart` - Siege arms use start
- `ReqSiegeArmsUseEnd` - Siege arms use end
- `ReqSiegeArmsRotationStart` - Siege arms rotation start
- `ReqSiegeArmsRotationStop` - Siege arms rotation stop
- `ReqSiegeArmsUseSkill` - Siege arms use skill
- `ReqSetPresentPassword` - Set present password
- `ReqPresentPasswordStatus` - Get present password status
- `ReqCheckPresentPassword` - Check present password
- `ReqGetAccountGift` - Get account gift
- `ReqAccountGiftList` - Get account gift list
- `ReqImprintGuardianSeal` - Imprint guardian seal
- `ReqConfirmImprintGuardianSealResult` - Confirm imprint result
- `ReqOpenCloseRangeChatRoom` - Open/close range chat room
- `ReqExtendCacheItem` - Extend cache item
- `ReqExtendCacheChangeItem` - Extend cache change item
- `ReqPreViewCacheItem` - Preview cache item
- `ReqCloseWebMall` - Close web mall

### Answer Messages (Ans)
- `AnsCheatTool` - Cheat tool response
- `AnsGmCommand` - GM command response
- `AnsAbilValue` - Ability value response
- `AnsInitSimulationStat` - Init simulation stat response
- `AnsLoadQuestTraceList` - Load quest trace list response
- `AnsLoadKharaTraceList` - Load khara trace list response
- `AnsPCMoveStateChange` - PC move state change
- `AnsUpdatePCPos` - Update PC position response
- `AnsRanking` - Ranking response
- `AnsRankingList` - Ranking list response
- `AnsRankerList` - Ranker list response
- `AnsRankerBuff` - Ranker buff response
- `AnsHelperDeclare` - Helper declare response
- `AnsCutSceneStart` - Cutscene start response
- `AnsCutSceneEnd` - Cutscene end response
- `AnsJobMasteryStarter` - Job mastery starter response
- `AnsJobMasteryTraining` - Job mastery training response
- `AnsJobMasteryUpgrade` - Job mastery upgrade response
- `AnsRegRevivalMercenary` - Revival mercenary response
- `AnsPCollection` - Pet collection response
- `AnsPCollectionSelectedSkill` - Pet collection skill
- `AnsPCollectionSetActivePet` - Set active pet response
- `AnsPCollectionIncubator` - Incubator response
- `AnsPCollectionUsePetEgg` - Use pet egg response
- `AnsPCollectionCombinEggBlank` - Combine egg blank response
- `AnsMCollection` - Monster collection response
- `AnsMCollectionSelectedSkill` - Monster collection skill
- `AnsMCollectionCombinProtein` - Combine protein response
- `AnsMCollectionUseProtein` - Use protein response
- `AnsThemeCostumeCompose` - Theme costume compose response
- `AnsSiegeArmsUseStart` - Siege arms use start response
- `AnsSiegeArmsUseEnd` - Siege arms use end response
- `AnsSiegeArmsUseSkill` - Siege arms use skill response
- `AnsSetPresentPassword` - Set present password response
- `AnsPresentPasswordStatus` - Present password status
- `AnsCheckPresentPassword` - Check present password response
- `AnsGetAccountGift` - Get account gift response
- `AnsImprintGuardianSeal` - Imprint guardian seal response
- `AnsConfirmImprintGuardianSealResult` - Confirm imprint result
- `AnsOpenCloseRangeChatRoom` - Open/close range chat room response
- `AnsMultiTeleportNpc` - Multi-teleport NPC response
- `AnsGliderRideScene` - Glider ride scene response
- `AnsTeleportSavePoint` - Teleport save point response
- `AnsRideRidingPet` - Ride pet response
- `AnsChangeProJobInfo` - Change pro job info
- `AnsEndScene` - End scene response
- `AnsExtendCacheItem` - Extend cache item response
- `AnsExtendCacheChangeItem` - Extend cache change item response
- `AnsPreViewCacheItem` - Preview cache item response

### Notify Messages (Nfy)
- `NfyServerTime` - Server time notification
- `NfyServerTimeToLoginPC` - Server time to login PC
- `NfyKharaTitleChange` - Khara title change
- `NfyUpdateQuestCompleted` - Quest completed update
- `NfyUpdateKharaCompleted` - Khara completed update
- `NfyDeleteAcceptedKhara` - Delete accepted khara
- `NfyPCMoveStateChange` - PC move state change
- `NfyOpenCloseRangeChatRoom` - Open/close range chat room
- `NfyJoinCloseRangeChatRoom` - Join range chat room
- `NfyExitCloseRangeChatRoom` - Exit range chat room
- `NfyNewMasterCloseRangeChatRoom` - New master range chat room
- `NfyUpdateJoinMemberCountCloseRangeChatRoom` - Update join member count
- `NfyRankingTopList` - Ranking top list
- `NfyRankerChange` - Ranker change
- `NfyHelperNotice` - Helper notice
- `NfyCutSceneStart` - Cutscene start notification
- `NfyJobMasteryPointChange` - Job mastery point change
- `NfyPushingStart` - Pushing start
- `NfyYouAreStressed` - You are stressed notification
- `NfyRevivalMercenaryState` - Revival mercenary state
- `NfyPCollection` - Pet collection notification
- `NfyPCollectionSkill` - Pet collection skill
- `NfyPCollectionSelectedSkill` - Pet collection selected skill
- `NfyMCollection` - Monster collection notification
- `NfyMCollectionSkill` - Monster collection skill
- `NfyMCollectionSelectedSkill` - Monster collection selected skill
- `NfySiegeStart` - Siege start
- `NfySiegeStartBeforehandStart` - Siege beforehand start
- `NfySiegeStartBeforehandRestart` - Siege beforehand restart
- `NfySiegeOccupy` - Siege occupy
- `NfySiegeStatusInfo` - Siege status info
- `NfySiegeHPStatus` - Siege HP status
- `NfySiegeArmsSettingChanged` - Siege arms setting changed
- `NfySiegeArmsUserChanged` - Siege arms user changed
- `NfySiegeArmsRotationStart` - Siege arms rotation start
- `NfySiegeArmsRotationStop` - Siege arms rotation stop
- `NfySiegeGuildTime` - Siege guild time
- `NfySiegeUnitPosition` - Siege unit position
- `NfyNpcBattleLevelUp` - NPC battle level up
- `NfyAccountGiftInfo` - Account gift info
- `NfyAccountGiftReceived` - Account gift received
- `NfyAccountGiftCount` - Account gift count
- `NfyNpcChat` - NPC chat
- `NfyNpcEffect` - NPC effect
- `NfyTargetInfo` - Target info
- `NfyAggroChanged` - Aggro changed
- `NfyHShieldErrorDetected` - HackShield error detected
- `NfyExcuteBuffHandling` - Execute buff handling
- `NfyPlayScene` - Play scene
- `NfySwimmingGuage` - Swimming gauge
- `NfyOpenRemoteShop` - Open remote shop
- `NfyOpenRemoteBank` - Open remote bank
- `NfyFuncObjectControlStatus` - Func object control status
- `NfyGraInternetCafeGradeChanged` - Internet cafe grade changed
- `NfyTPlusInfo` - T-Plus info
- `NfyPCCafeRevivalInfo` - PC Cafe revival info
- `NfySpinelRevivalInfo` - Spinel revival info
- `NfyRideRidingPet` - Ride riding pet
- `NfyDropRidingPet` - Drop riding pet

---

## Summary Statistics

- **Request (Req) Messages**: ~201
- **Answer (Ans) Messages**: ~201
- **Notify (Nfy) Messages**: ~201
- **Acknowledgment (Ack) Messages**: ~57

**Total Unique Messages**: ~660+

---

**Note:** This catalog was extracted from client strings. Actual message IDs, payloads, and detailed structures require packet capture analysis and further reverse engineering. Some messages may be deprecated or unused in the current game version.
