use crate::db;
use crate::models::*;
use rusqlite::Connection;

pub fn seed_demo(conn: &Connection) -> Result<Project, String> {
    let existing = db::list_projects(conn)?;
    if let Some(found) = existing.into_iter().find(|p| p.title == "烬城夜行") {
        return Ok(found);
    }

    let project = db::upsert_project(
        conn,
        Project {
            id: String::new(),
            title: "烬城夜行".into(),
            genre: "都市奇幻".into(),
            description: "雨夜的烬城底层，旧契约仍在呼吸。失去夜视的巡灯人沈照，被档案员白疏拽进一场不该被记录的失踪。".into(),
            style_prompt: "冷而克制，短句多，少解释。城市潮湿、灯火脏、对话有潜台词。避免喊口号和穿越式吐槽。".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_character(
        conn,
        Character {
            id: String::new(),
            project_id: project.id.clone(),
            name: "沈照".into(),
            aliases: "巡灯人,瞎子沈".into(),
            role: "主角 / 前巡灯司巡夜官".into(),
            personality: "寡言，记账式思维，恨麻烦但停不下来。对 lumina 过敏般厌恶。".into(),
            appearance: "右眼覆一层薄翳，常戴旧毡帽。左手小指缺一节。穿洗白的青灰风衣。".into(),
            background: "三年前在残灯会后门失去左眼的夜视能力，被巡灯司除名，现以代写文书为生。".into(),
            goals: "找到当年夜契被改写的人，并确认自己是否还算活人。".into(),
            relationships: "白疏是他唯一肯主动联系的人；对残灯会旧人保持礼貌的敌意。".into(),
            current_state: "刚收到一封没有署名的灰信，约他今夜去南桥下。右眼开始能看见灯芯里的字。".into(),
            notes: "说话不叫人全名，常用职业或外号。不用感叹号。".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_character(
        conn,
        Character {
            id: String::new(),
            project_id: project.id.clone(),
            name: "白疏".into(),
            aliases: "小白,档案室的".into(),
            role: "档案员 / 引路人".into(),
            personality: "语速快，爱纠正用词，害怕潮湿却总在雨里出门。表面刻薄，记仇也记恩。".into(),
            appearance: "短发，耳后一颗痣。常把铅笔别在领口，指腹有墨。".into(),
            background: "烬城地下档案库编外人员，能看见被墨水吞掉的旧记录。".into(),
            goals: "把被抹去的失踪者重新写回簿册，哪怕只写一行。".into(),
            relationships: "把沈照当半成品证人；对残灯会账房心怀旧怨。".into(),
            current_state: "偷出一页未归档的夜契残片，上面有沈照已烧掉的左眼签名。".into(),
            notes: "对话里爱用反问。不直说担心。".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_entry(
        conn,
        Entry {
            id: String::new(),
            project_id: project.id.clone(),
            title: "烬城".into(),
            category: "地点".into(),
            content: "沿河而建的旧工业城。白日灰扑扑，夜里灯火像未燃尽的炭。南桥以北是巡灯司旧辖区，以南是残灯会的地面。雨水会把墙上的告示冲成另一种句子。".into(),
            tags: "城市,雨,河".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_entry(
        conn,
        Entry {
            id: String::new(),
            project_id: project.id.clone(),
            title: "夜契".into(),
            category: "规则".into(),
            content: "用灯芯灰拌墨写成的契约。签下后，签名者在夜间必须履行条款，白天可以假装忘记。撕毁夜契不会取消义务，只会让义务改写到最近的见证人身上。".into(),
            tags: "契约,灯,规则".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_entry(
        conn,
        Entry {
            id: String::new(),
            project_id: project.id.clone(),
            title: "残灯会".into(),
            category: "组织".into(),
            content: "表面上是南桥旧灯具行会，实际买卖未完成的夜契。账房不记账本，只记账人。后门永远潮湿，门槛下埋着第一盏被熄灭的灯。".into(),
            tags: "组织,反派,南桥".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    let o1 = db::upsert_outline(
        conn,
        Outline {
            id: String::new(),
            project_id: project.id.clone(),
            parent_id: None,
            title: "卷一 · 灰信".into(),
            content: "沈照被一封灰信唤到南桥。白疏出现，出示夜契残片。两人必须在天亮前确认见证人是谁。".into(),
            order_index: 1,
            status: "writing".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    let o_ch1 = db::upsert_outline(
        conn,
        Outline {
            id: String::new(),
            project_id: project.id.clone(),
            parent_id: Some(o1.id.clone()),
            title: "第一章 · 南桥下".into(),
            content: "雨。灰信。沈照到得比约定早。桥洞里有一盏不该亮的灯。白疏把残片塞进他手里，残片烫得像刚灭的炭。远处有人用职业称呼叫他。".into(),
            order_index: 1,
            status: "writing".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_outline(
        conn,
        Outline {
            id: String::new(),
            project_id: project.id.clone(),
            parent_id: Some(o1.id.clone()),
            title: "第二章 · 灯芯里的字".into(),
            content: "沈照右眼看见灯芯内的条款：见证人栏是空的。白疏坚持空栏比写错更危险。他们前往残灯会后门，却发现门槛下的灯已被挖走。".into(),
            order_index: 2,
            status: "draft".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_chapter(
        conn,
        Chapter {
            id: String::new(),
            project_id: project.id.clone(),
            outline_id: Some(o_ch1.id.clone()),
            title: "第一章 · 南桥下".into(),
            content: "雨先到。\n\n沈照把灰信折进帽檐，站在南桥第三根桥柱旁边。水声很旧，像有人在桥肚里反复清嗓子。他数过——从码头到桥洞要走二百四十步，今晚他走了二百三十八步，因为右眼突然能看见表盘上不存在的刻度。\n\n灯亮着。不该亮。\n\n桥洞里那盏灯没有灯罩，灯芯却立得很直，像一截不肯承认自己已死的骨头。沈照没有靠近。他只是把左手插进风衣口袋，摸到缺了一节的小指，确认自己还在。\n\n“巡灯人。”有人在雨里叫他，声音嫌弃得熟练，“你又早到。账上最讨厌早到的人。”\n\n白疏从桥阶上走下来，铅笔还别在领口，头发已经湿了。她把一张焦边残片按进他手里。纸是烫的。\n\n沈照低头。残片上有他的左眼——准确说，是一只被墨水吞到只剩轮廓的眼睛，旁边签着他三年前的名字。".into(),
            summary: "雨夜南桥，沈照赴灰信之约。桥洞灯不该亮。白疏送来带有他左眼签名的夜契残片。".into(),
            order_index: 1,
            status: "writing".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_memory(
        conn,
        Memory {
            id: String::new(),
            project_id: project.id.clone(),
            kind: "character".into(),
            title: "沈照失去夜视".into(),
            content: "三年前在残灯会后门，沈照失去左眼的夜视，被巡灯司除名。右眼近日开始能看见灯芯里的字。".into(),
            source: "设定".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    db::upsert_memory(
        conn,
        Memory {
            id: String::new(),
            project_id: project.id.clone(),
            kind: "world".into(),
            title: "夜契不可撕".into(),
            content: "撕毁夜契不会取消义务，只会把义务改写到最近的见证人身上。见证人栏为空比填错更危险。".into(),
            source: "条目".into(),
            created_at: String::new(),
            updated_at: String::new(),
        },
    )?;

    Ok(project)
}
