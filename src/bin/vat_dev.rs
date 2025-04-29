use vat::repository::Repository;

fn main(){
    let repository = Repository::load();
    dbg!(&repository);
}