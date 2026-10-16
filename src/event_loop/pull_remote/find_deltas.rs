
use std::{cmp::Ordering, iter::Peekable, fmt::Debug};

use redb::{AccessGuard, Range};

use crate::{database::{table::{Entry, State}, table_trait::{ReadTable, TableConst}}, event_loop::pull_remote::RemoteDeltas};



pub fn find_remote_deltas(remote_state: &impl ReadTable<State>, local_state: &impl ReadTable<State>) -> RemoteDeltas {

    let remote_state_iter = remote_state.iter().unwrap().peekable();
    let local_state_iter = local_state.iter().unwrap().peekable();

    find(remote_state_iter, local_state_iter, RemoteDeltas::default())
}






fn find(mut r_state_iter: StateIter, mut l_state_iter: StateIter, mut pulls: RemoteDeltas) -> RemoteDeltas {
  

    let r_guard = r_state_iter.peek().map(unwrap_result);
    let l_guard = l_state_iter.peek().map(unwrap_result);
    
    
    match PathOrdering::from(r_guard,l_guard) {
        PathOrdering::Equal { r_entry, l_entry, ..} if !r_entry.is_newer(&l_entry) => {
            return find(next(r_state_iter), next(l_state_iter), pulls);
        },
        PathOrdering::Equal { path, r_entry, .. } if r_entry.is_deleted() => { // Updated Remotely
            pulls.delete.push(path.into());
            return find(next(r_state_iter), next(l_state_iter), pulls);
        },  
        PathOrdering::Equal { path, .. } => {
             pulls.update.push(path.into());
             return find(next(r_state_iter), next(l_state_iter), pulls);
        },
        PathOrdering::Less{r_path} => { // Created Remotely
            pulls.update.push(r_path.into());
            return find(next(r_state_iter), l_state_iter, pulls);
        },
        PathOrdering::Greater{..} => {
            return find(r_state_iter, next(l_state_iter), pulls);
        },
        PathOrdering::End => {
            return pulls
        },
    }
}








enum PathOrdering<'a> {
    Equal {
        path: &'a str,
        r_entry: Entry,
        l_entry: Entry,
    },
    Less {
        r_path: &'a str,
    },
    Greater {
        l_path: &'a str,
    },
    End
}

impl PathOrdering<'_> {
    fn from<'b>(r: Option<&'b EntryKVGuard<'_>>, l: Option<&'b EntryKVGuard<'_>>) -> PathOrdering<'b> {
        let (r_guard, l_guard) = match (r, l) {
            (Some(remote), Some(local)) => (remote,local),
            (Some((remote_path,_)), None) => return PathOrdering::Less{r_path: remote_path.value()},
            (None, Some((local_path,_))) => return PathOrdering::Greater{l_path: local_path.value()},
            (None, None) => return PathOrdering::End,
        };
    
        let (remote_path, remote_entry) = r_guard;
        let remote_path = remote_path.value();
        
        let (local_path, local_entry) = l_guard;
        let local_path = local_path.value();
        
        match remote_path.cmp(local_path) {
            Ordering::Equal => PathOrdering::Equal { path: remote_path, r_entry: remote_entry.value(), l_entry:local_entry.value() },
            Ordering::Greater => PathOrdering::Greater { l_path: local_path },
            Ordering::Less => PathOrdering::Less { r_path: remote_path },
        }
    }
}



type StateIter<'a> = Peekable<Range<'a, &'static str, Entry>>;

fn next(mut iter: StateIter) -> StateIter {
    iter.next();
    iter
}

fn unwrap_result<T,E: Debug>(res: &Result<T,E>) -> &T {
    res.as_ref().unwrap()
}
type EntryKVGuard<'a> = (AccessGuard<'a, &'static str>, AccessGuard<'a, Entry>);

